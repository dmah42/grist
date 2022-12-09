mod object;
mod repo;

#[macro_use]
extern crate ini;

use crate::object::Commit;
use crate::repo::Repo;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::{collections::HashSet, path::PathBuf};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, ValueEnum)]
enum ObjectType {
    Blob,
    Commit,
    Tag,
    Tree,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
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
        #[arg(short, default_value_t = String::from("HEAD"))]
        commit: String,
    },
    LsTree {
        /// The tree object to list
        #[arg(short)]
        tree: String,
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
    let cwd = std::env::current_dir()?;
    let mut repo = Repo::find(&cwd).context("unable to find repo")?;
    let content = match type_ {
        ObjectType::Blob => object::Blob::read(&mut repo, object)
            .context(format!("failed to read blob {}", object))?
            .decode()?,
        //ObjectType::Commit => object::Commit::read(&mut repo, object)
        //    .context(format!("failed to read commit {}", object))?,
        ObjectType::Tag => String::from("UNIMPLEMENTED TAG CAT"),
        //ObjectType::Tree => object::Tree::read(&mut repo, object)
        //    .context(format!("failed to read tree {}", object))?,
        _ => String::from("unimplemented cat for {type_}"),
    };
    println!("{}", content);
    Ok(())
}

fn hash_object(write: bool, type_: &Option<ObjectType>, file: &PathBuf) -> Result<()> {
    log::info!("hashing {:?} {:?}", type_, file);
    let cwd = std::env::current_dir()?;

    log::debug!("finding worktree from {:?}", cwd);

    let mut repo = Repo::find(&cwd).context("unable to find repo")?;
    let content = std::fs::read(file).context(format!("failed to read file {file:?}"))?;
    let hash = object::hash(&content);

    let content_str =
        String::from_utf8(content).context("failed to convert content of file from utf8")?;

    if write {
        match type_ {
            None | Some(ObjectType::Blob) => {
                object::Blob::write(&mut repo, &hash, &object::Blob::new(&content_str))
            }
            _ => bail!("unimplemented hash for {type_:?}"),
        }
    }
    println!("{}", hash);
    Ok(())
}

fn init(path: &Option<PathBuf>) -> Result<Repo> {
    log::info!("initializing grist repo at {:?}", path);
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let repo_path = path.as_ref().unwrap_or(&cwd);
    Repo::create(repo_path)
}

fn log_graphviz(commit: &String) -> Result<()> {
    log::info!("logging from {}", commit);
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let mut repo = Repo::find(&cwd).context("unable to find repo")?;

    let mut seen: HashSet<String> = HashSet::new();

    log_relationship(&mut repo, commit, &mut seen)
}

fn log_relationship(repo: &mut Repo, hash: &String, seen: &mut HashSet<String>) -> Result<()> {
    if seen.contains(hash) {
        return Ok(());
    }
    seen.insert(hash.clone());

    let commit = Commit::read(repo, hash).context("failed to read commit {hash}")?;

    let parents = &commit.parents;

    for p in parents {
        println!("c{} -> c{}", hash, p);
    }
    Ok(())
}

fn object_type(hash: &String) -> Result<Option<ObjectType>> {
    let cwd = std::env::current_dir()?;
    let mut repo = Repo::find(&cwd).context("unable to find repo")?;
    if let Ok(_blob) = object::Blob::read(&mut repo, hash) {
        return Ok(Some(ObjectType::Blob));
    } else if let Ok(_commit) = object::Commit::read(&mut repo, hash) {
        return Ok(Some(ObjectType::Commit));
    } else if let Ok(_tree) = object::Tree::read(&mut repo, hash) {
        return Ok(Some(ObjectType::Tree));
    }
    Ok(None)
}

fn lstree(hash: &String) -> Result<()> {
    log::info!("listing tree {}", hash);
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let mut repo = Repo::find(&cwd).context("unable to find repo")?;

    let tree = object::Tree::read(&mut repo, hash)?;

    for item in tree.items {
        println!(
            "{} {:?} {}\t{:?}",
            item.mode,
            object_type(&item.hash)?,
            item.hash,
            item.path
        )
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    match &args.command {
        Commands::Add { path } => match add(path) {
            Ok(_) => println!("added {:?}", path),
            Err(error) => bail!("failed to add {path:?}: {error}"),
        },
        Commands::CatFile { type_, object } => {
            cat_file(type_, object)
                .context(format!("failed to cat file {:?} {}", type_, object))?;
        }
        Commands::HashObject { write, type_, file } => {
            hash_object(write.unwrap_or(false), type_, file)
                .context(format!("failed to hash object {:?} {:?}", type_, file))?;
        }
        Commands::Init { path } => match init(path) {
            Ok(_) => println!("initialized repo at {:?}", path),
            Err(error) => bail!("failed to initialize repo at {path:?}: {error}"),
        },
        Commands::Log { commit } => {
            log_graphviz(commit)?;
        }
        Commands::LsTree { tree } => {
            lstree(tree)?;
        }
        _ => bail!("unknown command: {:?}", &args.command),
    }
    Ok(())
}
