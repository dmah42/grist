use crate::repo::Repo;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::PathBuf;

pub(crate) fn hash(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Blob {
    /// hex encoded blob content
    hex: String,
}

impl Blob {
    pub(crate) fn new(content: &String) -> Self {
        Blob {
            hex: hex::encode(content),
        }
    }

    pub(crate) fn decode(&self) -> Result<String> {
        let decoded = hex::decode(self.hex.clone())?;
        String::from_utf8(decoded).context("failed to convert {decoded} to utf8")
    }

    // TODO: generic trait for read/write.
    pub(crate) fn read(repo: &mut Repo, hash: &String) -> Result<Self> {
        if let Some(blob) = repo.blobs().read().get(hash) {
            log::debug!("hex: {}; hex_len: {}", blob.hex, blob.hex.len());
            Ok(blob.clone())
        } else {
            Err(anyhow!("hash {hash} not found"))
        }
    }

    pub(crate) fn write(repo: &mut Repo, hash: &str, blob: &Blob) {
        log::debug!("hex: {}; hex_len: {}", blob.hex, blob.hex.len());
        repo.blobs().write().insert(hash.to_owned(), blob.clone());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Author {
    name: String,
    email: String,
    // TODO: timestamp?
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Commit {
    /// hex string
    pub tree: String,

    /// hex strings
    pub parents: Vec<String>,

    pub author: Author,
    pub committer: Author,

    // TODO: optional gpgsig
    pub comment: String,
}

impl Commit {
    pub(crate) fn read(repo: &mut Repo, hash: &String) -> Result<Self> {
        if let Some(commit) = repo.commits().read().get(hash) {
            log::debug!("commit: {:?}", commit);
            Ok(commit.clone())
        } else {
            Err(anyhow!("{hash} not found"))
        }
    }

    pub(crate) fn write(repo: &mut Repo, hash: &str, commit: &Commit) {
        repo.commits()
            .write()
            .insert(hash.to_owned(), commit.clone());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TreeItem {
    pub mode: String,
    pub path: PathBuf,
    // hex string referencing tree (directory) or blob (file)
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Tree {
    pub items: Vec<TreeItem>,
}

impl Tree {
    pub(crate) fn read(repo: &mut Repo, hash: &String) -> Result<Tree> {
        if let Some(tree) = repo.trees().read().get(hash) {
            log::debug!("tree: {tree:?}");
            Ok(tree.clone())
        } else {
            Err(anyhow!("{hash} not found"))
        }
    }

    pub(crate) fn write(repo: &mut Repo, hash: &str, tree: &Tree) {
        repo.trees().write().insert(hash.to_owned(), tree.clone());
    }
}
