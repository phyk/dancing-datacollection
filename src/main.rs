use anyhow::{Context, Result};
use clap::Parser;
use dancing_datacollection::scraper::{Config, Scraper};
use env_logger::Env;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let config_content = fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config file: {}", args.config))?;
    let config: Config =
        toml::from_str(&config_content).with_context(|| "Failed to parse config TOML")?;

    let mut scraper = Scraper::new();
    scraper.scrape_all(&config).await?;

    Ok(())
}
