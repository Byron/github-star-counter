#![feature(async_closure)]
#![feature(async_await)]

#[macro_use]
#[cfg(test)]
extern crate lazy_static;

use futures::future::join_all;
use itertools::Itertools;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::{future::Future, io};

type Error = Box<dyn std::error::Error>;

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

struct BasicAuth {
    username: String,
    password: Option<String>,
}

async fn request<D>(url: &str, auth: Option<BasicAuth>) -> Result<D, Error>
// TODO want Result<impl DeserializeOwned, ...> but that does not compile
where
    D: DeserializeOwned,
{
    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder()
        .build::<_, hyper::Body>(https);
    let mut request = client.get(&format!("https://api.github.com/{}", url));
    if let Some(auth) = auth {
        request = request.basic_auth(auth.username, auth.password);
    }
    let mut r: Response = request.send().await;
    if r.status().is_success() {
        r.json();
    } else {
        r.json();
    }
    unimplemented!()
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
