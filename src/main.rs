mod repo;

#[macro_use]
extern crate ini;

use crate::repo::Repo;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { path: std::path::PathBuf },
    Checkout {},
    Commit {},
    Init { path: Option<std::path::PathBuf> },
    Log {},
    Merge {},
    Rebase {},
    Rm {},
    Tag {},
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    match &args.command {
        Commands::Add { path } => {
            log::info!("adding {:?}", path);
        }
        Commands::Init { path } => {
            log::info!("initializing grist repo at {:?}", path);
            let cwd_or_err = std::env::current_dir();
            if cwd_or_err.is_err() {
                panic!("failed to get current directory");
            }
            let cwd = cwd_or_err.unwrap();
            let repo_path = path.as_ref().unwrap_or(&cwd);
            match Repo::create(repo_path) {
                Ok(_) => println!("created repo at {:?}", repo_path),
                Err(error) => log::error!("failed to create repo at {:?}: {}", repo_path, error),
            }
        }
        _ => panic!("command"),
    }
}
