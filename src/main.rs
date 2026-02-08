use anyhow::Result;
use clap::Parser;
use dancing_datacollection::crawler::client::Scraper;
use env_logger::Env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Download {
        /// URL to scrape
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    match args.command {
        Commands::Download { url } => {
            let mut scraper = Scraper::new();
            scraper.scrape_all(&[url]).await?;
        }
    }

    Ok(())
}
