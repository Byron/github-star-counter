mod options;

extern crate lazy_static;
use github_star_counter::{render_output, count_stars, BasicAuth, Error, Repo};
use options::RequestUser;
use simple_logger;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    use options::Args;
    let args: Args = Args::from_args();
    simple_logger::init_with_level(args.log_level).ok();
    let auth: Option<BasicAuth> = get_auth(args.auth, args.username.clone());

    let response = count_stars(&args.username, args.no_orgs, auth, args.page_size).await?;
    let user_login = response.user.login;

    let repos: Vec<Repo> = response.repos;
    let output = render_output(
        args.template,
        repos,
        user_login,
        args.repo_limit,
        args.stargazer_threshold,
    )?;
    println!("{}", output);
    Ok(())
}

fn get_auth(auth: RequestUser, username: String) -> Option<BasicAuth> {
    match (auth.request_username, auth.request_password) {
        (Some(username), password) => Some(BasicAuth { username, password }),
        (None, Some(password)) => Some(BasicAuth {
            username: username,
            password: Some(password),
        }),
        _ => None,
    }
}
