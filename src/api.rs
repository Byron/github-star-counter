use crate::request::BasicAuth;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Repo {
    pub stargazers_count: usize,
    pub name: String,
    pub owner: RepoOwner,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct RepoStats {
    pub total: usize,
    pub total_by_user_only: Vec<usize>,
    pub total_by_orgs_only: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct RepoOwner {
    pub login: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub login: String,
    pub public_repos: usize,
}

pub struct Options {
    pub no_orgs: bool,
    pub auth: Option<BasicAuth>,
    pub page_size: usize,
    pub repo_limit: usize,
    pub stargazer_threshold: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            auth: None,
            no_orgs: false,
            page_size: 100,
            repo_limit: 10,
            stargazer_threshold: 0,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub user: User,
    pub repos: Vec<Repo>,
}
