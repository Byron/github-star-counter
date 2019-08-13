#![feature(async_closure)]
#![feature(async_await)]

#[macro_use]
#[cfg(test)]
extern crate lazy_static;

use base64;
use futures::future::join_all;
use hyper;
use itertools::Itertools;
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

impl ToString for BasicAuth {
    fn to_string(&self) -> String {
        format!(
            "Basic {}",
            base64::encode(&match &self.password {
                Some(password) => format!("{}:{}", self.username, password),
                None => self.username.clone(),
            })
        )
    }
}

async fn request_json<D>(url: &str, auth: Option<BasicAuth>) -> Result<D, Error>
// TODO want Result<impl DeserializeOwned, ...> but that does not compile
where
    D: DeserializeOwned,
{
    use hyper::{Body, Client, Request, Response};
    async fn request_body_into_string(body: Response<Body>) -> Result<Vec<u8>, Error> {
        let mut body = body.into_body();
        let mut out = Vec::new();
        while let Some(chunk) = body.next().await {
            let chunk = chunk?;
            out.extend_from_slice(chunk.as_ref());
        }
        Ok(out)
    };
    let https = hyper_tls::HttpsConnector::new(1)?;
    let client = Client::builder().build::<_, Body>(https);

    let mut req: Request<_> = Request::new(Body::empty());
    req.headers_mut()
        .append("User-Agent", "GitHub StarCounter.rs".parse()?);
    if let Some(auth) = auth {
        req.headers_mut()
            .append("Authorization", auth.to_string().parse()?);
    }
    *req.uri_mut() = format!("https://api.github.com/{}", url)
        .parse()
        .expect("valid URL");

    let mut res = client.request(req).await?;
    let status = res.status();
    let bytes = request_body_into_string(res).await?;

    if status.is_success() {
        Ok(serde_json::from_slice(&bytes)?)
    } else {
        #[derive(Deserialize)]
        struct Error {
            message: String,
        }
        let err: Error = serde_json::from_slice(&bytes).or_else(|e| {
            Ok::<_, serde_json::Error>(Error {
                message: format!(
                    "Unexpected error message format returned by Github: '{:#?}'",
                    e
                ),
            })
        })?;
        Err(err.message.into())
    }
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
