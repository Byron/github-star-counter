#![feature(async_closure)]

#[macro_use]
extern crate lazy_static;
pub use crate::request::BasicAuth;
use bytesize::ByteSize;
use futures::future::join_all as join_all_futures;
use futures::{FutureExt, TryFutureExt};
use itertools::Itertools;
use log::{error, info};
use std::{future::Future, sync::atomic::Ordering, time::Instant};

mod api;
mod request;

pub use crate::api::*;

pub type Error = Box<dyn std::error::Error>;

pub async fn count_stars(
    username: &str,
    no_orgs: bool,
    auth: Option<BasicAuth>,
    page_size: usize,
) -> Result<Response, Error> {
    let fetch_repos_for_user = |user| {
        fetch_repos(user, page_size, |user, page_number| {
            let repos_paged_url = format!(
                "users/{}/repos?per_page={}&page={}",
                user.login,
                page_size,
                page_number + 1
            );
            request::json_log_failure(repos_paged_url, auth.clone())
        })
        .map_err(|e| {
            error!("Could not fetch repositories: {}", e);
            e
        })
    };
    let flatten_into_vec = |vec: Vec<_>| vec.into_iter().flatten().flatten().collect::<Vec<_>>();

    let user_url = format!("users/{}", username);
    let user: User = request::json(user_url.clone(), auth.clone()).await?;
    let orgs_url = format!("{}/orgs", user_url);
    let mut user_repos_futures = vec![fetch_repos_for_user(user.clone()).boxed_local()];

    if !no_orgs {
        let auth = auth.clone();
        let orgs_repos_future = async move {
            let orgs: Vec<RepoOwner> = request::json_log_failure(orgs_url, auth.clone())
                .await
                .unwrap_or_else(|_| Vec::new());

            let repos_of_orgs = flatten_into_vec(
                join_all_futures(orgs.into_iter().map(|user| {
                    request::json_log_failure::<User>(format!("users/{}", user.login), auth.clone())
                        .and_then(fetch_repos_for_user)
                }))
                .await,
            );
            Ok(repos_of_orgs)
        }
        .boxed_local();
        user_repos_futures.push(orgs_repos_future);
    };

    let start = Instant::now();
    let repos = flatten_into_vec(join_all_futures(user_repos_futures).await);

    let elapsed = start.elapsed();
    let duration_in_network_requests = request::TOTAL_DURATION.lock().unwrap().as_secs_f32();
    info!(
        "Total bytes received in body: {}",
        ByteSize(request::TOTAL_BYTES_RECEIVED_IN_BODY.load(Ordering::Relaxed))
    );
    info!(
        "Total time spent in network requests: {:.2}s",
        duration_in_network_requests
    );
    info!(
        "Wallclock time for future processing: {:.2}s",
        elapsed.as_secs_f32()
    );
    info!(
        "Speedup due to networking concurrency: {:.2}x",
        duration_in_network_requests / elapsed.as_secs_f32()
    );

    Ok(Response { user, repos })
}

async fn fetch_repos<F>(
    user: User, // TODO: can this also be &User?
    page_size: usize,
    mut fetch_page: impl FnMut(User, usize) -> F, // TODO would want 'async impl' for -> F; and &User instead of User!
) -> Result<Vec<Repo>, Error>
where
    F: Future<Output = Result<Vec<Repo>, Error>>,
{
    if page_size == 0 {
        return Err("PageSize must be greater than 0".into());
    }
    let page_count = user.public_repos / page_size;
    let page_futures = (0..=page_count).map(|page_number| fetch_page(user.clone(), page_number));
    let results = join_all_futures(page_futures).await;
    let pages_with_results: Vec<Vec<Repo>> = results
        .into_iter()
        .collect::<Result<Vec<Vec<_>>, Error>>()?
        .into_iter()
        .collect();

    sanity_check(page_size, &pages_with_results);
    Ok(pages_with_results.into_iter().concat())
}

#[cfg(test)]
fn sanity_check(_page_size: usize, _pages_with_results: &Vec<Vec<Repo>>) {}

#[cfg(not(test))]
fn sanity_check(page_size: usize, pages_with_results: &Vec<Vec<Repo>>) {
    if pages_with_results.len() > 0 {
        if let Some(v) = pages_with_results
            .iter()
            .take(
                pages_with_results
                    .len()
                    .checked_sub(1)
                    .expect("more than one page"),
            )
            .filter(|v| v.len() != page_size)
            .next()
        {
            panic!(
                "Asked for {} repos per page, but got only {} in a page which wasn't the last one. --page-size should probably be {}",
                page_size,
                v.len(),
                v.len()
            );
        }
    }
}

#[cfg(test)]
mod tests;
