mod options;

use github_star_counter::{count_stars, BasicAuth, Error, Repo};
use options::RequestUser;
use simple_logger;
use std::io;
use std::io::stdout;
use std::path::PathBuf;
use structopt::StructOpt;
use tera::{Context, Tera};
use std::fs;

fn filter_repos(repos: &Vec<Repo>, user_login: String, is_user: bool) -> Vec<usize> {
    let compare_username_matches = |want: bool, user: String| {
        move |r: &Repo| {
            if r.owner.login.eq(&user) == want {
                Some(r.stargazers_count)
            } else {
                None
            }
        }
    };

    repos
        .iter()
        .filter_map(compare_username_matches(is_user, user_login.clone()))
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    use options::Args;
    let args: Args = Args::from_args();
    simple_logger::init_with_level(args.log_level).ok();
    let stargazer_threshold = args.stargazer_threshold;

    let auth: Option<BasicAuth> = get_auth(args.auth, args.username.clone());

    let response = count_stars(&args.username, args.no_orgs, auth, args.page_size).await?;
    let user_login = response.user.login;

    let mut repos: Vec<_> = response
        .repos
        .into_iter()
        .filter(|r| r.stargazers_count >= stargazer_threshold)
        .take(args.repo_limit)
        .collect();

    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));

    output(args.template, repos, user_login.clone())
}

fn output(template: Option<PathBuf>, mut repos: Vec<Repo>, login: String) -> Result<(), Error> {
    let total: usize = repos.iter().map(|r| r.stargazers_count).sum();
    let total_by_user_only = filter_repos(&repos, login.clone(), true);
    let total_by_orgs_only = filter_repos(&repos, login.clone(), false);

    if !total_by_orgs_only.is_empty() {
        for mut repo in repos.iter_mut() {
            repo.name = format!("{}/{}", repo.owner.login, repo.name);
        }
    }

    match template {
        Some(template) => template_output(
            repos,
            total,
            total_by_user_only,
            total_by_orgs_only,
            login,
            template,
        ),
        None => default_output(
            repos,
            total,
            total_by_user_only,
            total_by_orgs_only,
            login,
            stdout(),
        ),
    }
}

fn template_output(
    repos: Vec<Repo>,
    total: usize,
    total_by_user_only: Vec<usize>,
    total_by_orgs_only: Vec<usize>,
    login: String,
    template: PathBuf,
) -> Result<(), Error> {
    let mut context = Context::new();
    context.insert("repos", &repos);
    context.insert("total", &total);
    context.insert("total_by_user_only", &total_by_user_only);
    context.insert("total_by_orgs_only", &total_by_orgs_only);
    context.insert("login", &login);

    let template: String = fs::read_to_string(template)?;
    let rendered = Tera::one_off(&template, &context, true)?;
    println!("{}", rendered);
    Ok(())
}

fn default_output(
    repos: Vec<Repo>,
    total: usize,
    total_by_user_only: Vec<usize>,
    total_by_orgs_only: Vec<usize>,
    login: String,
    mut out: impl io::Write,
) -> Result<(), Error> {
    writeln!(out, "Total: {}", total)?;
    if !total_by_user_only.is_empty() && !total_by_orgs_only.is_empty() {
        writeln!(
            out,
            "Total for {}: {}",
            login,
            total_by_user_only.iter().sum::<usize>()
        )?;
    }
    if !total_by_orgs_only.is_empty() {
        writeln!(
            out,
            "Total for orgs: {}",
            total_by_orgs_only.iter().sum::<usize>()
        )?;
    }

    if repos.len() > 0 {
        writeln!(out)?;
    }

    let max_width = repos.iter().map(|r| r.name.len()).max().unwrap_or(0);
    for repo in repos {
        writeln!(
            out,
            "{:width$}   ★  {}",
            repo.name,
            repo.stargazers_count,
            width = max_width
        )?;
    }
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
