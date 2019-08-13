#![feature(async_await)]
use github_star_counter::{count_stars, Error, Options};
use std::io::stdout;
use structopt::StructOpt;

mod options;

#[tokio::main(single_thread)]
async fn main() -> Result<(), Error> {
    use options::Args;

    let args: Args = Args::from_args();
    let name = args.username.clone();
    count_stars(&name, stdout(), args.into()).await
}
