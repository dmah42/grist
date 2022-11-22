use acidjson::AcidJson;
use configparser::ini::Ini;
use simple_error::SimpleError;
use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

type ConfigMap = HashMap<String, HashMap<String, Option<String>>>;
type BlobsTable = AcidJson<HashMap<String, String>>;

const VERSION: &str = "0";

#[derive(Debug)]
pub(crate) struct Db {
    blobs: BlobsTable,
}

pub(crate) struct Repo {
    worktree: PathBuf,
    config: ConfigMap,
    db: Db,
}

impl<'a> Repo {
    // TODO: create a "force" version instead of requiring [force] to be passed in.
    fn new(worktree: &Path, force: bool) -> Result<Self, Box<dyn Error>> {
        let gristpath = worktree.join(".grist");

        log::debug!("checking if {:?} is a dir", gristpath);
        if !force && !gristpath.is_dir() {
            return Err(Box::new(SimpleError::new("not a gristpath")));
        }

        // create config if it doesn't exist
        log::debug!("creating/reading config");
        let config_path = gristpath.join("config.yaml");

        let config = match config_path.exists() {
            true => Some(ini!(config_path.to_str().unwrap())),
            false => match force {
                true => Some(ConfigMap::new()),
                false => None,
            },
        };

        if config.is_none() {
            return Err(Box::new(SimpleError::new("config not found")));
        }

        if !force
            && config.as_ref().unwrap()["core"]["repositoryformatversion"]
                != Some(String::from(VERSION))
        {
            return Err(Box::new(SimpleError::new("unsupported repo version")));
        }

        let dbpath = gristpath.join("db");
        // create the database
        log::debug!("creating database in {:?}", dbpath);
        let mut blobpath = dbpath.to_path_buf();
        blobpath.push("blobs.json");
        if std::fs::read(&blobpath).is_err() {
            std::fs::write(&blobpath, b"{}")?;
        }

        let blobs = AcidJson::open(blobpath.as_path())?;

        Ok(Self {
            worktree: worktree.to_path_buf(),
            config: config.unwrap(),
            db: Db { blobs },
        })
    }

    /// Create a new repo at [path].
    pub(crate) fn create(path: &Path) -> Result<Self, Box<dyn Error>> {
        log::debug!("creating repo at {:?}", path);
        let repo = Repo::new(path, true)?;
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
    /// returns a [Repo] if found, and [None] if not unless [required] is true,
    /// in which case it returns an [Error].
    pub(crate) fn find(path: &Path, required: bool) -> Result<Option<Repo>, Box<dyn Error>> {
        log::debug!("checking {:?} is a worktree", path);
        if path.join(".grist").is_dir() {
            log::debug!("it is!");
            return Ok(Some(Repo::new(path, false)?));
        }
        let Some(parent) = path.parent() else {
            if required {
                Err(Box::new(SimpleError::new("no grist directory")))
            } else {
                Ok(None)
            }
        };
        Self::find(parent, required)
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
