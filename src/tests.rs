// #[tokio::test]
// async fn fetch_all_repos_paged() {
//     use std::sync::atomic::{AtomicUsize, Ordering};
//     let mut repos_twice: Vec<_> = REPOS.clone();
//     repos_twice.extend_from_slice(&REPOS);
//     let mut user: User = USER.clone();
//     user.public_repos = repos_twice.len();
//     let page_size = 100;
//     let fetch_page_calls = AtomicUsize::new(0);

//     // FETCH with paging
//     {
//         let fetch_page_calls = &fetch_page_calls;
//         let fetch_page = async move |user: User, _page: usize| {
//             fetch_page_calls.fetch_add(1, Ordering::Acquire);
//             Ok(if user.login == "Byron" {
//                 REPOS.clone()
//             } else {
//                 Vec::new()
//             })
//         };

//         assert_eq!(
//             fetch_repos(user, page_size, fetch_page).await.unwrap(),
//             repos_twice
//         );
//     }
//     assert_eq!(fetch_page_calls.load(Ordering::Relaxed), 2);
// }
