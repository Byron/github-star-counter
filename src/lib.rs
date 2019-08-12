#[macro_use]
#[cfg(test)]
extern crate lazy_static;

use itertools::Itertools;
use serde::Deserialize;
use std::io;

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

fn fetch_repos(
    user: &User,
    page_size: usize,
    mut fetch_page: impl FnMut(&User, usize) -> Result<Vec<Repo>, Error>,
) -> Result<Vec<Repo>, Error> {
    if page_size == 0 {
        return Err("PageSize must be greater than 0".into());
    }
    let page_count = user.public_repos / page_size;
    Ok((0..=page_count)
        .map(|page_number| fetch_page(user, page_number))
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
