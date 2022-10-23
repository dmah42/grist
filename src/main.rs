mod object;
mod repo;

#[macro_use]
extern crate ini;

use crate::repo::Repo;
use clap::{Parser, Subcommand, ValueEnum};
use simple_error::SimpleError;
use std::{error::Error, path::PathBuf};

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

        file: PathBuf,
    },
    Init {
        path: Option<PathBuf>,
    },
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

fn cat_file(type_: &ObjectType, object: &String) -> Result<(), Box<dyn Error>> {
    log::info!("catting {:?} {}", type_, object);
    let cwd_or_err = std::env::current_dir();
    let cwd = cwd_or_err.unwrap();
    let mut repo = Repo::find(&cwd, true)?.unwrap();
    let content = match type_ {
        ObjectType::Blob => object::Blob::read(&mut repo, object)?,
        ObjectType::Commit => String::from("UNIMPLEMENTED COMMIT CAT"),
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
) -> Result<(), Box<dyn Error>> {
    log::info!("hashing {:?} {:?}", type_, file);
    let cwd_or_err = std::env::current_dir();
    let cwd = cwd_or_err.unwrap();
    let repo = Repo::find(&cwd, write)?;

    let content = std::fs::read(file)?;

    let hash = object::hash(&content);

    if write {
        match type_.unwrap() {
            ObjectType::Blob => object::Blob::write(&mut repo.unwrap(), &hash, &content)?,
            _ => Err(SimpleError::new(format!(
                "unimplemented hash_object for type {:?}",
                type_
            )))?,
        }
    }
    println!("{}", hash);
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
        Commands::CatFile { type_, object } => {
            let res = cat_file(type_, object);
            if res.is_err() {
                log::error!(
                    "failed to cat file {:?} {}: {}",
                    type_,
                    object,
                    res.err().unwrap()
                );
            }
        }
        Commands::HashObject { write, type_, file } => {
            let res = hash_object(write.unwrap_or(false), type_, file);
            if res.is_err() {
                log::error!(
                    "failed to hash object {:?} {:?}: {}",
                    type_,
                    file,
                    res.err().unwrap()
                );
            }
        }
        Commands::Init { path } => match init(path) {
            Ok(_) => println!("initialized repo at {:?}", path),
            Err(error) => log::error!("failed to initialize repo at {:?}: {}", path, error),
        },
        _ => panic!("command"),
    }
}
