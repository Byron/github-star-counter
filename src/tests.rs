use super::*;
use pretty_assertions::assert_eq;
use serde_json;

static USERNAME: &str = "Byron";
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
    output(USERNAME, REPOS.clone(), 10, 0, &mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), USER_OUTPUT);
}

#[test]
fn output_repos_with_threshold() {
    let mut buf = Vec::new();
    output(USERNAME, REPOS.clone(), 10, 30, &mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), USER_OUTPUT_THRESHOLD_30);
}

#[tokio::test]
async fn fetch_all_repos_paged() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let mut repos_twice: Vec<_> = REPOS.clone();
    repos_twice.extend_from_slice(&REPOS);
    let mut user: User = USER.clone();
    user.public_repos = repos_twice.len();
    let page_size = 100;
    let fetch_page_calls = AtomicUsize::new(0);

    // FETCH with paging
    {
        let fetch_page_calls = &fetch_page_calls;
        let fetch_page = async move |user: User, _page: usize| {
            fetch_page_calls.fetch_add(1, Ordering::Acquire);
            Ok(if user.login == "Byron" {
                REPOS.clone()
            } else {
                Vec::new()
            })
        };

        assert_eq!(
            fetch_repos(user, page_size, fetch_page).await.unwrap(),
            repos_twice
        );
    }
    assert_eq!(fetch_page_calls.load(Ordering::Relaxed), 2);
}
