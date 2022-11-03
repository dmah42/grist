use configparser::ini::Ini;
use gluesql::{prelude::Glue, sled_storage::SledStorage};
use simple_error::SimpleError;
use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

type ConfigMap = HashMap<String, HashMap<String, Option<String>>>;

const VERSION: &str = "0";

pub(crate) struct Repo {
    worktree: PathBuf,
    config: ConfigMap,
    db: Glue<SledStorage>,
}

impl<'a> Repo {
    // TODO: create a "force" version instead of requiring [force] to be passed in.
    pub(crate) fn new(worktree: &Path, force: bool) -> Result<Self, Box<dyn Error>> {
        let gristdir = worktree.join(".grist");

        log::debug!("checking if {:?} is a dir", gristdir);
        if !force && !gristdir.is_dir() {
            return Err(Box::new(SimpleError::new("not a gristdir")));
        }

        // create config if it doesn't exist
        log::debug!("creating/reading config");
        let config_path = gristdir.join("config.yaml");

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

        // finding database
        let db_path = gristdir.join("db.sled");
        let db_url = db_path.to_str().unwrap();

        let storage = SledStorage::new(db_url)?;
        let mut db = Glue::new(storage);

        Ok(Self {
            worktree: worktree.to_path_buf(),
            config: config.unwrap(),
            db,
        })
    }

    /// Create a new repo at [path].
    pub(crate) fn create(path: &Path) -> Result<Self, Box<dyn Error>> {
        log::debug!("creating repo at {:?}", path);
        let mut repo = Repo::new(path, true)?;
        let gristdir = &repo.gristdir();

        log::debug!("gristdir: {:?}", gristdir);

        if gristdir.exists() {
            log::debug!("already exists");
            if !gristdir.is_dir() {
                return Err(Box::new(SimpleError::new(format!(
                    "{:?} is not a directory",
                    gristdir
                ))));
            }
        } else {
            log::debug!("creating gristdir");
            std::fs::create_dir(gristdir)?;
        }

        // create the database
        log::debug!("creating database");

        repo.db.execute(
            "CREATE TABLE \
            Blobs ( \
                hash TEXT PRIMARY KEY,
                content TEXT,
            );",
        )?;

        // TODO: this is where we'd replace filesystem with databases
        std::fs::create_dir_all(gristdir.join("branches"))?;
        //std::fs::create_dir(gristdir.join("objects"))?;
        std::fs::create_dir_all(gristdir.join("refs").join("tags"))?;
        std::fs::create_dir_all(gristdir.join("refs").join("heads"))?;

        std::fs::write(
            gristdir.join("description"),
            "unnamed repo: edit this file to name the repo",
        )?;
        std::fs::write(gristdir.join("HEAD"), "ref: refs/heads/master\n")?;

        Self::default_config().write(gristdir.join("config.yaml").to_str().unwrap())?;

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

    pub(crate) fn gristdir(&self) -> PathBuf {
        self.worktree.join(".grist")
    }

    pub(crate) fn db(&mut self) -> &mut Glue<SledStorage> {
        &mut self.db
    }
}
