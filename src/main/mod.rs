mod options;

#[macro_use]
extern crate lazy_static;
use github_star_counter::{count_stars, BasicAuth, Error, Repo, RepoStats};
use options::RequestUser;
use simple_logger;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use tera::{Context, Tera};

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

fn get_stats(repos: &Vec<Repo>, login: String) -> RepoStats {
    let total: usize = repos.iter().map(|r| r.stargazers_count).sum();
    let total_by_user_only = filter_repos(&repos, login.clone(), true);
    let total_by_orgs_only = filter_repos(&repos, login.clone(), false);

    RepoStats {
        total,
        total_by_user_only,
        total_by_orgs_only,
    }
}

fn render_output(
    template: Option<PathBuf>,
    mut repos: Vec<Repo>,
    login: String,
    repo_limit: usize,
    stargazer_threshold: usize,
) -> Result<String, Error> {
    let stats = get_stats(&repos, login.to_string());

    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
    let mut repos: Vec<_> = repos
        .into_iter()
        .filter(|r| r.stargazers_count >= stargazer_threshold)
        .take(repo_limit)
        .collect();

    if !stats.total_by_orgs_only.is_empty() {
        for mut repo in repos.iter_mut() {
            repo.name = format!("{}/{}", repo.owner.login, repo.name);
        }
    }

    match template {
        Some(template) => template_output(
            repos,
            stats,
            login,
            template,
        ),
        None => default_output(
            repos,
            stats,
            login,
        ),
    }
}

pub fn template_output(
    repos: Vec<Repo>,
    stats: RepoStats,
    login: String,
    template: PathBuf,
) -> Result<String, Error> {
    let mut context = Context::new();
    context.insert("repos", &repos);
    context.insert("total", &stats.total);
    context.insert("total_by_user_only", &stats.total_by_user_only);
    context.insert("total_by_orgs_only", &stats.total_by_orgs_only);
    context.insert("login", &login);

    let template: String = fs::read_to_string(template)?;
    let rendered = Tera::one_off(&template, &context, true)?;
    Ok(rendered)
}

pub fn default_output(
    repos: Vec<Repo>,
    stats: RepoStats,
    login: String,
) -> Result<String, Error> {
    let mut out = String::new();
    writeln!(out, "Total: {}", stats.total)?;
    if !stats.total_by_user_only.is_empty() && !stats.total_by_orgs_only.is_empty() {
        writeln!(
            out,
            "Total for {}: {}",
            login,
            stats.total_by_user_only.iter().sum::<usize>()
        )?;
    }
    if !stats.total_by_orgs_only.is_empty() {
        writeln!(
            out,
            "Total for orgs: {}",
            stats.total_by_orgs_only.iter().sum::<usize>()
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
    Ok(out)
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

#[cfg(test)]
mod tests {
    use super::*;
    use github_star_counter::User;
    use pretty_assertions::assert_eq;
    use serde_json;

    static USERNAME: &str = "Byron";
    static USER_JSON: &str = include_str!("../../test/fixtures/github.com-byron.json");
    static PAGE1_JSON: &str =
        include_str!("../../test/fixtures/github.com-byron-repos-page-1.json");
    static USER_OUTPUT: &str = include_str!("../../test/fixtures/github.com-byron-output.txt");
    static USER_OUTPUT_THRESHOLD_30: &str =
        include_str!("../../test/fixtures/github.com-byron-output-threshold-30.txt");
    static TEMPLATE_OUTPUT: &str = include_str!("../../test/fixtures/template_output.md");

    lazy_static! {
        static ref USER: User = serde_json::from_str(USER_JSON).unwrap();
        static ref REPOS: Vec<Repo> = serde_json::from_str(PAGE1_JSON).unwrap();
    }
    #[test]
    fn output_repos() {
        let output = render_output(None, REPOS.clone(), USERNAME.to_string(), 10, 0).unwrap();
        assert_eq!(output, USER_OUTPUT);
    }

    #[test]
    fn output_repos_with_threshold() {
        let output = render_output(None, REPOS.clone(), USERNAME.to_string(), 10, 30).unwrap();
        assert_eq!(output, USER_OUTPUT_THRESHOLD_30);
    }

    #[test]
    fn output_repos_with_custom_template() {
        let output = render_output(
            Some(PathBuf::from("test/fixtures/template.md")),
            REPOS.clone(),
            USERNAME.to_string(),
            10,
            30,
        )
        .unwrap();
        assert_eq!(output, TEMPLATE_OUTPUT);
    }
}
