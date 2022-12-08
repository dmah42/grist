mod object;
mod repo;

#[macro_use]
extern crate ini;

use crate::repo::Repo;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, ValueEnum)]
enum ObjectType {
    Blob,
    Commit,
    Tag,
    Tree,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        path: std::path::PathBuf,
    },
    CatFile {
        #[arg(value_enum)]
        type_: ObjectType,
        /// hash of the object to cat
        object: String,
    },
    Checkout {},
    Commit {},
    HashObject {
        #[arg(short, action = clap::ArgAction::SetTrue)]
        write: Option<bool>,

        #[arg(short, value_enum)]
        type_: Option<ObjectType>,

        /// path to file to be hashed
        file: PathBuf,
    },
    Init {
        path: Option<PathBuf>,
    },
    Log {
        /// The commit at which to start the log
        #[arg(short, default_value_t = "HEAD")]
        commit: Option<String>,
    },
    Merge {},
    Rebase {},
    Rm {},
    Tag {},
}

fn add(path: &PathBuf) -> Result<()> {
    log::info!("adding {:?}", path);
    Ok(())
}

fn cat_file(type_: &ObjectType, object: &String) -> Result<()> {
    log::info!("catting {:?} {}", type_, object);
    let cwd = std::env::current_dir()?.unwrap();
    let mut repo = Repo::find(&cwd).context("unable to find repo")?.unwrap();
    let content = match type_ {
        ObjectType::Blob => object::Blob::read(&mut repo, object).context(format!("failed to read blob {}", object))?,
        ObjectType::Commit => object::Blob::read(&mut repo, object).context(format!("failed to read commit {}", object))?,
        ObjectType::Tag => String::from("UNIMPLEMENTED TAG CAT"),
        ObjectType::Tree => String::from("UNIMPLEMENTED TREE CAT"),
    };
    println!("{}", content);
    Ok(())
}

fn hash_object(
    write: bool,
    type_: &Option<ObjectType>,
    file: &PathBuf,
) -> Result<()> {
    log::info!("hashing {:?} {:?}", type_, file);
    let cwd = std::env::current_dir()?.unwrap();

    log::debug!("finding worktree from {:?}", cwd);

    let repo = Repo::find(&cwd).context("unable to find repo")?;
    let content = std::fs::read(file).context(format!("failed to read file {}", file))?;
    let hash = object::hash(&content);

    if write {
        match type_ {
            None | Some(ObjectType::Blob) => {
                object::Blob::write(&mut repo.unwrap(), &hash, &content)
            },
            _ => Err(SimpleError::new(format!(
                "unimplemented hash_object for type {:?}",
                type_
            )))?,
        }
    }
    println!("{}", hash);
    Ok(())
}

fn init(path: &Option<PathBuf>) -> Result<Repo> {
    log::info!("initializing grist repo at {:?}", path);
    let cwd = std::env::current_dir().context("failed to get current directory")?.unwrap();
    let repo_path = path.as_ref().unwrap_or(&cwd);
    Repo::create(repo_path)
}

fn log_graphviz(commit: String) -> Result<()> {
    log::info!("logging from {}", commit);
    let cwd = std::env::current_dir().context("failed to get current directory")?.unwrap();
    let repo = Repo::find(&cwd).context("unable to find repo")?;

    let mut seen = HashSet::new();

    return log_relationship(&repo, hash, &seen)
}

fn log_relationship(repo: &Repo, hash: String, seen: &mut HashSet) -> Result<()> {
    if seen.contains(hash) {
        Ok(())
    }
    seen.insert(hash);

    commit = Commit::read(repo, hash)?;

    let parents = &commit.parents;

    for p in parents {
        println!("c{} -> c{}", hash, p);
    }

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    match &args.command {
        Commands::Add { path } => match add(path) {
            Ok(_) => println!("added {:?}", path),
            Err(error) => return error.context(format!("failed to add {:?}: {}", path, error)),
        },
        Commands::CatFile { type_, object } => {
            cat_file(type_, object).context(format!("failed to cat file {:?} {}", type_, object))?;
        }
        Commands::HashObject { write, type_, file } => {
            hash_object(write.unwrap_or(false), type_, file).context(format!("failed to hash object {:?} {:?}", type_, file))?;
        }
        Commands::Init { path } => match init(path) {
            Ok(_) => println!("initialized repo at {:?}", path),
            Err(error) => return error.context(format!("failed to initialize repo at {:?}: {}", path, error)),
        },
        Commands::Log { commit } => {
            log_graphviz(commit)?
        }
        _ => return Err(format!("unknown command {}", &args.command)),
    }
    Ok(())
}
