mod repo;

#[macro_use]
extern crate ini;

use crate::repo::Repo;
use clap::{Parser, Subcommand};
use std::{error::Error, path::PathBuf};

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
    Init { path: Option<PathBuf> },
    Log {},
    Merge {},
    Rebase {},
    Rm {},
    Tag {},
}

fn add(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    log::info!("adding {:?}", path);
    Ok(())
}

fn init(path: &Option<PathBuf>) -> Result<Repo, Box<dyn Error>> {
    log::info!("initializing grist repo at {:?}", path);
    let cwd_or_err = std::env::current_dir();
    if cwd_or_err.is_err() {
        panic!("failed to get current directory");
    }
    let cwd = cwd_or_err.unwrap();
    let repo_path = path.as_ref().unwrap_or(&cwd);
    Repo::create(repo_path)
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    match &args.command {
        Commands::Add { path } => match add(path) {
            Ok(_) => println!("added {:?}", path),
            Err(error) => log::error!("failed to add {:?}: {}", path, error),
        },
        Commands::Init { path } => match init(path) {
            Ok(_) => println!("initialized repo at {:?}", path),
            Err(error) => log::error!("failed to initialize repo at {:?}: {}", path, error),
        },
        _ => panic!("command"),
    }
}
