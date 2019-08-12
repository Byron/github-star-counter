#[macro_use]
#[cfg(test)]
extern crate lazy_static;

use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Clone))]
struct User {
    pub public_repos: usize,
}

fn fetch_repos(
    user: &User,
    page_size: usize,
    mut fetch_page: impl FnMut(&User, usize) -> Vec<Repo>,
) -> Result<Vec<Repo>, Box<dyn Error>> {
    if page_size == 0 {
        return Err("PageSize must be greater than 0".into());
    }
    let page_count = user.public_repos / page_size;
    Ok((0..=page_count).fold(Vec::new(), |mut acc, page_number| {
        acc.append(&mut fetch_page(user, page_number));
        acc
    }))
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
    use serde_json;
    static USER_JSON: &str = include_str!("../test/fixtures/github.com-byron.json");
    static PAGE1_JSON: &str = include_str!("../test/fixtures/github.com-byron-repos-page-1.json");

    lazy_static! {
        static ref USER: User = serde_json::from_str(USER_JSON).unwrap();
        static ref REPOS: Vec<Repo> = serde_json::from_str(PAGE1_JSON).unwrap();
    }

    #[test]
    fn fetch_all_repos_paged() {
        let mut repos_twice: Vec<_> = REPOS.clone();
        repos_twice.extend_from_slice(&REPOS);
        let mut user: User = USER.clone();
        user.public_repos = repos_twice.len();
        const PAGE_SIZE: usize = 100;
        let mut fetch_page_callcount = 0;
        {
            let fetch_page = |_user: &User, _page: usize| {
                fetch_page_callcount += 1;
                REPOS.clone()
            };

            assert_eq!(
                repos_twice,
                fetch_repos(&user, PAGE_SIZE, fetch_page).unwrap()
            );
        }
        assert_eq!(fetch_page_callcount, 2);
    }
}
