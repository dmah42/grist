use configparser::ini::Ini;
use simple_error::SimpleError;
use std::collections::HashMap;

type ConfigMap = HashMap<String, HashMap<String, Option<String>>>;

static VERSION: &str = "0";

pub(crate) struct Repo {
    worktree: std::path::PathBuf,
    config: ConfigMap,
}

impl Repo {
    pub(crate) fn new(worktree: &std::path::Path, force: bool) -> Result<Self, SimpleError> {
        let gristdir = worktree.join(".grist");

        if !force && !gristdir.is_dir() {
            return Err(SimpleError::new("not a gristdir"));
        }

        let config_path = gristdir.join("config");

        let config = match config_path.exists() {
            true => Some(ini!(config_path.to_str().unwrap())),
            false => match force {
                true => Some(ConfigMap::new()),
                false => None,
            },
        };

        if config.is_none() {
            return Err(SimpleError::new("config not found"));
        }

        if !force
            && config.as_ref().unwrap()["core"]["repositoryformatversion"]
                != Some(String::from(VERSION))
        {
            return Err(SimpleError::new("unsupported repo version"));
        }

        Ok(Self {
            worktree: worktree.to_path_buf(),
            config: config.unwrap(),
        })
    }

    /// Create a new repo at [path].
    pub(crate) fn create(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
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

        let gristdir = worktree.join(".grist");

        std::fs::create_dir_all(gristdir.join("branches"))?;
        std::fs::create_dir(gristdir.join("objects"))?;
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
}
