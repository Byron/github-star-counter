use super::Options;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Aggregate your repositories' stars in GitHub!")]
pub struct Args {
    /// The amount of repositories per page when asking for your repository details
    #[structopt(short = "p", long = "page-size", default_value = "50")]
    pub page_size: usize,
    /// The amount of repositories to displays at most. Set it to 0 to only see your total stars
    #[structopt(short = "r", long = "repo-limit", default_value = "10")]
    pub repo_limit: usize,
    /// The amount of stars a repository should have at the least to be considered for the repository list.
    /// Note that this does not affect your total star count.
    /// If 0, all repositories are considered.
    #[structopt(short = "s", long = "stargazer-threshold", default_value = "0")]
    pub stargazer_threshold: usize,
    /// The name of the github user, like "Byron"
    pub username: String,
}

impl From<Args> for Options {
    fn from(
        Args {
            repo_limit,
            stargazer_threshold,
            page_size,
            ..
        }: Args,
    ) -> Self {
        Options {
            page_size,
            repo_limit,
            stargazer_threshold,
            ..Options::default()
        }
    }
}
