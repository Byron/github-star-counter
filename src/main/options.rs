use super::Options;
use github_star_counter::BasicAuth;
use structopt::StructOpt;
use tracing::Level;

#[derive(Debug, StructOpt)]
#[structopt(about = "Aggregate your repositories' stars in GitHub!")]
pub struct Args {
    /// If set, organizations one is a member of will not be taken into consideration.
    /// This speeds up the query, but is less precise.
    #[structopt(long = "no-orgs")]
    pub no_orgs: bool,
    /// The amount of repositories per page when asking for your repository details
    #[structopt(short = "p", long = "page-size", default_value = "100")]
    pub page_size: usize,
    /// The amount of repositories to displays at most. Set it to 0 to only see your total stars
    #[structopt(short = "r", long = "repo-limit", default_value = "10")]
    pub repo_limit: usize,
    /// The desired log level. Only 'INFO' is implemented right now to provide timing information.
    #[structopt(short = "l", long = "log-level", default_value = "ERROR")]
    #[structopt(raw(possible_values = r#"&["INFO", "ERROR", "DEBUG"]"#))]
    pub log_level: Level,
    /// The amount of stars a repository should have at the least to be considered for the repository list.
    /// Note that this does not affect your total star count.
    /// If 0, all repositories are considered.
    #[structopt(short = "s", long = "stargazer-threshold", default_value = "1")]
    pub stargazer_threshold: usize,
    #[structopt(flatten)]
    pub auth: RequestUser,
    /// The name of the github user, like "Byron"
    pub username: String,
}

#[derive(Debug, StructOpt)]
pub struct RequestUser {
    /// The name of the user to use for authenticated requests against the API.
    /// Use this if you run into issues with github API usage limits
    #[structopt(short = "u", long = "request-username")]
    request_username: Option<String>,
    /// The password of the user to use for authenticated requests against the API.
    /// Use this if you run into issues with github API usage limits
    /// Be sure to prefix the whole command with a single space to prevent it from
    /// getting stored in your shell history file. Please note this might not work
    /// in your particular shell.
    /// If only the password is provided, the user for which stars are counted is used
    /// as --request-username
    #[structopt(long = "request-password")]
    request_password: Option<String>,
}

impl From<Args> for Options {
    fn from(
        Args {
            no_orgs,
            repo_limit,
            stargazer_threshold,
            page_size,
            auth,
            username,
            ..
        }: Args,
    ) -> Self {
        Options {
            no_orgs,
            page_size,
            repo_limit,
            stargazer_threshold,
            auth: match (auth.request_username, auth.request_password) {
                (Some(username), password) => Some(BasicAuth { username, password }),
                (None, Some(password)) => Some(BasicAuth {
                    username,
                    password: Some(password),
                }),
                _ => None,
            },
        }
    }
}
