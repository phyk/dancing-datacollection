use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Download, validate and store competition results (same pipeline as the Python API)
    Download {
        /// URL of a competition or event index page
        url: String,
        /// Base directory for results and raw files
        #[arg(long, default_value = "data")]
        target_folder: PathBuf,
        /// Date filter/override (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// Age group filter (e.g. "Adult", "Sen I")
        #[arg(long)]
        age_group: Option<String>,
        /// Style filter ("Standard" or "Latein")
        #[arg(long)]
        style: Option<String>,
        /// Level filter (e.g. "D", "C", "B", "A", "S")
        #[arg(long)]
        level: Option<String>,
        /// Do not archive raw HTML files
        #[arg(long)]
        no_html: bool,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    match args.command {
        Commands::Download {
            url,
            target_folder,
            date,
            age_group,
            style,
            level,
            no_html,
        } => {
            let filters = dancing_datacollection::parse_filters(
                date.as_deref(),
                age_group.as_deref(),
                style.as_deref(),
                level.as_deref(),
            )
            .map_err(anyhow::Error::msg)?;
            let saved =
                dancing_datacollection::run_pipeline(&target_folder, &url, &filters, !no_html)?;
            log::info!("Saved {} competition(s) to {:?}", saved, target_folder);
        }
    }

    Ok(())
}
