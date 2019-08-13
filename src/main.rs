#![feature(async_await)]

use github_star_counter::{count_stars, Error, Options};
use std::io::stdout;

#[tokio::main(single_thread)]
async fn main() -> Result<(), Error> {
    count_stars("Byron", stdout(), Options::default()).await
}
