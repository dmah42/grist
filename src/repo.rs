use crate::object::Commit;
use acidjson::AcidJson;
use anyhow::Result;
use configparser::ini::Ini;
use simple_error::SimpleError;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

type ConfigMap = HashMap<String, HashMap<String, Option<String>>>;
type BlobsTable = AcidJson<HashMap<String, String>>;
type CommitsTable = AcidJson<HashMap<String, Commit>>;

const VERSION: &str = "0";

#[derive(Debug)]
pub(crate) struct Db {
    blobs: BlobsTable,
    commits: CommitsTable,
}

pub(crate) struct Repo {
    worktree: PathBuf,
    config: ConfigMap,
    db: Db,
}

impl<'a> Repo {
    fn new(worktree: &Path) -> Result<Self> {
        let gristpath = worktree.join(".grist");

        // create config if it doesn't exist
        log::debug!("creating/reading config");
        let config_path = gristpath.join("config.yaml");

        let config = if config_path.exists() {
            ini!(config_path.to_str().unwrap())
        } else {
            ConfigMap::new()
        };

        let dbpath = gristpath.join("db");
        // create the database
        log::debug!("creating database in {:?}", dbpath);
        let mut blobpath = dbpath.clone();
        blobpath.push("blobs.json");
        if std::fs::read(&blobpath).is_err() {
            std::fs::write(&blobpath, b"{}")?;
        }

        let blobs = AcidJson::open(blobpath.as_path())?;

        let mut commitpath = dbpath;
        commitpath.push("commits.json");
        if std::fs::read(&commitpath).is_err() {
            std::fs::write(&commitpath, b"{}")?;
        }
        let commits = AcidJson::open(commitpath.as_path())?;

        Ok(Self {
            worktree: worktree.to_path_buf(),
            config,
            db: Db { blobs, commits },
        })
    }

    fn load(worktree: &Path) -> Result<Self> {
        let gristpath = worktree.join(".grist");

        log::debug!("checking if {:?} is a dir", gristpath);
        if !gristpath.is_dir() {
            return Err(Box::new(SimpleError::new("not a gristpath")));
        }

        // create config if it doesn't exist
        log::debug!("creating/reading config");
        let config_path = gristpath.join("config.yaml");

        let config = if config_path.exists() {
            ini!(config_path.to_str().unwrap())
        } else {
            return Err(Box::new(SimpleError::new("config not found")));
        };

        if config["core"]["repositoryformatversion"] != Some(String::from(VERSION)) {
            return Err(Box::new(SimpleError::new("unsupported repo version")));
        }

        let dbpath = gristpath.join("db");
        log::debug!("loading database from {:?}", dbpath);
        let mut blobpath = dbpath.to_path_buf();
        blobpath.push("blobs.json");

        let blobs = AcidJson::open(blobpath.as_path())?;

        let mut commitpath = dbpath.to_path_buf();
        commitpath.push("blobs.json");

        let commits = AcidJson::open(blobpath.as_path())?;

        Ok(Self {
            worktree: worktree.to_path_buf(),
            config,
            db: Db { blobs, commits },
        })
    }

    /// Create a new repo at [path].
    pub(crate) fn create(path: &Path) -> Result<Self> {
        log::debug!("creating repo at {:?}", path);
        let repo = Repo::new(path)?;
        let gristpath = &repo.gristpath();

        log::debug!("gristdir: {:?}", gristpath);

        if gristpath.exists() {
            log::debug!("already exists");
            if !gristpath.is_dir() {
                return Err(Box::new(SimpleError::new(format!(
                    "{:?} is not a directory",
                    gristpath
                ))));
            }
        } else {
            log::debug!("creating gristdir");
            std::fs::create_dir(gristpath)?;
        }

        // TODO: this is where we'd replace filesystem with databases
        std::fs::create_dir_all(gristpath.join("branches"))?;
        //std::fs::create_dir(gristpath.join("objects"))?;
        std::fs::create_dir_all(gristpath.join("refs").join("tags"))?;
        std::fs::create_dir_all(gristpath.join("refs").join("heads"))?;

        std::fs::write(
            gristpath.join("description"),
            "unnamed repo: edit this file to name the repo",
        )?;
        std::fs::write(gristpath.join("HEAD"), "ref: refs/heads/master\n")?;

        Self::default_config().write(gristpath.join("config.yaml").to_str().unwrap())?;

        Ok(repo)
    }

    /// recursively walk up from [path] to find a [Repo].
    /// returns a [Repo] if found, or an [Error] if not.
    pub(crate) fn find(path: &Path) -> Result<Repo> {
        log::debug!("checking {:?} is a worktree", path);
        if path.join(".grist").is_dir() {
            log::debug!("it is!");
            return Ok(Repo::load(path)?);
        }
        let Some(parent) = path.parent() else {
            return Err(Box::new(SimpleError::new("no grist directory")));
        };
        Self::find(parent)
    }

    fn default_config() -> Ini {
        let mut config = Ini::new();
        config.set(
            "core",
            "repositoryformatversion",
            Some(String::from(VERSION)),
        );
        config.set("core", "filemode", Some(false.to_string()));
        config.set("core", "bare", Some(false.to_string()));
        config
    }

    pub(crate) fn worktree(&'a self) -> &'a Path {
        &self.worktree
    }

    pub(crate) fn gristpath(&self) -> PathBuf {
        self.worktree.join(".grist")
    }

    pub(crate) fn dbpath(&self) -> PathBuf {
        self.gristpath().join("db")
    }

    pub(crate) fn blobs(&self) -> &BlobsTable {
        &self.db().blobs
    }

    pub(crate) fn db(&self) -> &Db {
        &self.db
    }
}
