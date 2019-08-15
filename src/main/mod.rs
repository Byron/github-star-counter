#![feature(async_await)]
use github_star_counter::{count_stars, Error, Options};
use std::io::stdout;
use structopt::StructOpt;
use tracing_fmt;

mod options;

#[tokio::main]
async fn main() -> Result<(), Error> {
    use options::Args;

    let args: Args = Args::from_args();
    let name = args.username.clone();
    let subscriber = tracing_fmt::FmtSubscriber::builder()
        .with_filter(tracing_fmt::filter::EnvFilter::from("async_fn=trace"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    count_stars(&name, stdout(), args.into()).await
}
