#![feature(async_closure)]
#![feature(async_await)]

#[macro_use]
#[cfg(test)]
extern crate lazy_static;

pub use crate::request::BasicAuth;
use futures::future::join_all;
use itertools::Itertools;
use serde::Deserialize;
use std::{future::Future, io};

mod request;

pub type Error = Box<dyn std::error::Error>;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Eq, PartialEq))]
struct Repo {
    stargazers_count: usize,
    name: String,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Clone))]
struct User {
    public_repos: usize,
}

pub struct Options {
    pub auth: Option<BasicAuth>,
    pub page_size: usize,
    pub repo_limit: usize,
    pub stargazer_threshold: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            auth: None,
            page_size: 100,
            repo_limit: 10,
            stargazer_threshold: 0,
        }
    }
}

pub async fn count_stars(
    username: &str,
    out: impl io::Write,
    Options {
        auth,
        page_size,
        repo_limit,
        stargazer_threshold,
    }: Options,
) -> Result<(), Error> {
    let user_url = format!("users/{}", username);
    let user: User = request::json(&user_url, auth.as_ref()).await?;

    // TODO make this into 'async' (without move) closure so we don't move these
    // It's strange that the move happening at the end is not allowed, it should be fine
    // to have the closure own these after they have been used.
    let user_url_closure = &user_url;
    let auth_closure = &auth;
    let repos = fetch_repos(&user, page_size, async move |_user, page_number| {
        let repos_paged_url = format!(
            "{}/repos?per_page={}&page={}",
            user_url_closure,
            page_size,
            page_number + 1
        );
        request::json(&repos_paged_url, auth_closure.as_ref()).await
    })
    .await?;
    output(repos, repo_limit, stargazer_threshold, out)
}

async fn fetch_repos<F>(
    user: &User,
    page_size: usize,
    mut fetch_page: impl FnMut(&User, usize) -> F, // TODO would want 'async impl'
) -> Result<Vec<Repo>, Error>
where
    F: Future<Output = Result<Vec<Repo>, Error>>,
{
    if page_size == 0 {
        return Err("PageSize must be greater than 0".into());
    }
    let page_count = user.public_repos / page_size;
    let futures = (0..=page_count).map(|page_number| fetch_page(user, page_number));
    let results: Vec<Result<Vec<Repo>, Error>> = join_all(futures).await;
    Ok(results
        .into_iter()
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .concat())
}

fn output(
    mut repos: Vec<Repo>,
    repo_limit: usize,
    stargazer_threshold: usize,
    mut out: impl io::Write,
) -> Result<(), Error> {
    let total: usize = repos.iter().map(|r| r.stargazers_count).sum();

    writeln!(out, "Total: {}", total)?;

    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
    let repos: Vec<_> = repos
        .iter()
        .filter(|r| r.stargazers_count >= stargazer_threshold)
        .take(repo_limit)
        .collect();
    let longest_name_len = repos.iter().map(|r| r.name.len()).max().unwrap_or(0);

    if repos.len() > 0 {
        writeln!(out)?;
    }
    for repo in repos {
        writeln!(
            out,
            "{:width$}   ★  {}",
            repo.name,
            repo.stargazers_count,
            width = longest_name_len
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
