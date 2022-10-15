use configparser::ini::Ini;
use diesel::{Connection, SqliteConnection};
use simple_error::SimpleError;
use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

type ConfigMap = HashMap<String, HashMap<String, Option<String>>>;

static VERSION: &str = "0";

pub(crate) struct Repo {
    worktree: PathBuf,
    config: ConfigMap,
    db: SqliteConnection,
}

impl<'a> Repo {
    // TODO: create a "force" version instead of requiring [force] to be passed in.
    pub(crate) fn new(worktree: &Path, force: bool) -> Result<Self, Box<dyn Error>> {
        let gristdir = worktree.join(".grist");

        if !force && !gristdir.is_dir() {
            return Err(Box::new(SimpleError::new("not a gristdir")));
        }

        let db_path = gristdir.join("db.sqlite3");
        let db_url = db_path.to_str().unwrap();
        let db = SqliteConnection::establish(db_url)?;

        let config_path = gristdir.join("config");

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

        Ok(Self {
            worktree: worktree.to_path_buf(),
            config: config.unwrap(),
            db,
        })
    }

    /// Create a new repo at [path].
    pub(crate) fn create(path: &Path) -> Result<Self, Box<dyn Error>> {
        let repo = Repo::new(path, true)?;
        let worktree = &repo.worktree;

        if worktree.exists() {
            if !worktree.is_dir() {
                return Err(Box::new(SimpleError::new(format!(
                    "{:?} is not a directory",
                    worktree
                ))));
            }
            if std::fs::read_dir(worktree)?.count() > 0 {
                return Err(Box::new(SimpleError::new(format!(
                    "{:?} is not empty",
                    worktree
                ))));
            }
        } else {
            std::fs::create_dir(worktree)?;
        }

        // TODO: this is where we'd replace filesystem with sqlite or similar
        let gristdir = worktree.join(".grist");

        std::fs::create_dir_all(gristdir.join("branches"))?;
        //std::fs::create_dir(gristdir.join("objects"))?;
        std::fs::create_dir_all(gristdir.join("refs").join("tags"))?;
        std::fs::create_dir_all(gristdir.join("refs").join("heads"))?;

        std::fs::write(
            gristdir.join("description"),
            "unnamed repo: edit this file to name the repo",
        )?;
        std::fs::write(gristdir.join("HEAD"), "ref: refs/heads/master\n")?;

        Self::default_config().write(gristdir.join("config").to_str().unwrap())?;

        Ok(repo)
    }

    /// recursively walk up from [path] to find a [Repo].
    /// returns a [Repo] if found, and [None] if not unless [required] is true,
    /// in which case it returns an [Error].
    pub(crate) fn find(path: &Path, required: bool) -> Result<Option<Repo>, Box<dyn Error>> {
        if path.join(".grist").is_dir() {
            return Ok(Some(Repo::new(path, false)?));
        }
        match path.parent() {
            Some(parent) => Self::find(parent, required),
            None => {
                if required {
                    Err(Box::new(SimpleError::new("no grist directory")))
                } else {
                    Ok(None)
                }
            }
        }
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

    pub(crate) fn db(&mut self) -> &mut SqliteConnection {
        &mut self.db
    }
}
