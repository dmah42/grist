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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match &args.command {
        Commands::Add { path } => {
            println!("adding {:?}", path);
        }
        Commands::Init { path } => {
            println!("initializing grist repo at {:?}", path);
            let cwd = std::env::current_dir()?;
            let repo_path = path.as_ref().unwrap_or(&cwd);
            Repo::create(repo_path)?;
        }
        _ => println!("oops"),
    }
    println!("Hello, world!");

    Ok(())
}
