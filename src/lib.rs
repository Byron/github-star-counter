#[macro_use]
#[cfg(test)]
extern crate lazy_static;

use itertools::Itertools;
use serde::Deserialize;
use std::io;

type Error = Box<dyn std::error::Error>;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Clone))]
struct User {
    pub public_repos: usize,
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
    if repo_limit > 0 {
        writeln!(out)?;
    }

    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
    let repo_iter = repos
        .iter()
        .filter(|r| r.stargazers_count >= stargazer_threshold);
    let longest_name = repo_iter.clone().map(|r| r.name.len()).max().unwrap_or(0);
    for repo in repo_iter.take(repo_limit) {
        writeln!(
            out,
            "{:width$}   ★  {}",
            repo.name,
            repo.stargazers_count,
            width = longest_name
        )?;
    }
    Ok(())
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, Eq, PartialEq))]
struct Repo {
    pub stargazers_count: usize,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json;

    static USER_JSON: &str = include_str!("../test/fixtures/github.com-byron.json");
    static PAGE1_JSON: &str = include_str!("../test/fixtures/github.com-byron-repos-page-1.json");
    static USER_OUTPUT: &str = include_str!("../test/fixtures/github.com-byron-output.txt");
    static USER_OUTPUT_THRESHOLD_30: &str =
        include_str!("../test/fixtures/github.com-byron-output-threshold-30.txt");

    lazy_static! {
        static ref USER: User = serde_json::from_str(USER_JSON).unwrap();
        static ref REPOS: Vec<Repo> = serde_json::from_str(PAGE1_JSON).unwrap();
    }
    #[test]
    fn output_repos() {
        let mut buf = Vec::new();
        output(REPOS.clone(), 10, 0, &mut buf).unwrap();

        assert_eq!(String::from_utf8(buf).unwrap(), USER_OUTPUT);
    }

    #[test]
    fn output_repos_with_threshold() {
        let mut buf = Vec::new();
        output(REPOS.clone(), 10, 30, &mut buf).unwrap();

        assert_eq!(String::from_utf8(buf).unwrap(), USER_OUTPUT_THRESHOLD_30);
    }

    #[test]
    fn fetch_all_repos_paged() {
        let mut repos_twice: Vec<_> = REPOS.clone();
        repos_twice.extend_from_slice(&REPOS);
        let mut user: User = USER.clone();
        user.public_repos = repos_twice.len();
        const PAGE_SIZE: usize = 100;
        let mut fetch_page_calls = 0;

        // FETCH with paging
        {
            let fetch_page = |_user: &User, _page: usize| {
                fetch_page_calls += 1;
                Ok(REPOS.clone())
            };

            assert_eq!(
                fetch_repos(&user, PAGE_SIZE, fetch_page).unwrap(),
                repos_twice
            );
        }
        assert_eq!(fetch_page_calls, 2);
    }
}
