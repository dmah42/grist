use crate::repo::Repo;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ObjectError {
    #[error("hash {hash} not found")]
    HashNotFound { hash: String },

    #[error("function unimplemented")]
    Unimplemented,

    #error(transparent)
    FromHexError{#[from] hex::FromHexError},

    #error(transparent)
    Utf8Error{#[from] Utf8Error},
}

pub(crate) fn hash(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub(crate) struct Blob {
    /// hex encoded blob content
    hex: [u8],
}

impl Blob {
    pub(crate) new(content: &String) -> Self  => {
        Blob {
            hex: hex::encode(content),
        }
    }

    pub(crate) fn decode(self: Self) -> String {
        String::from(hex::decode(self.hex)?)
    }

    // TODO: generic trait for read/write.
    pub(crate) fn read(repo: &mut Repo, hash: &String) -> Result<Self, ObjectError> {
        if let Some(blob) = repo.blobs().read().get(hash) {
            log::debug!("hex: {hex}; hex_len: {}", blob.hex.len());
            Ok(blob)
        } else {
            Err(HashNotFound{hash})
        }
    }

    pub(crate) fn write(repo: &mut Repo, hash: &String, blob: &Blob) {
        log::debug!("hex: {}; hex_len: {}", blob.hex, blob.hex.len());
        repo.blobs().write().insert(hash, blob);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Author {
    name: String,
    email: String,
    // TODO: timestamp?
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Commit {
    /// hex string
    tree: String,

    /// hex strings
    parents: Vec<String>,

    author: Author,
    committer: Author,

    // TODO: optional gpgsig

    comment: String,
}

impl Commit {
    pub(crate) fn read(repo: &mut Repo, sha: &String) -> Result<Self> {
        if let Some(commit) = repo.blobs().read().get(hash) {
            log::debug!("commit: {commit}");
            Ok(commit)
        } else {
            Err(HashNotFound{hash})
        }
    }

    pub(crate) fn write(repo: &mut Repo, hash: &String, commit: &Commit) {
        repo.blobs().write().insert(hash, commit);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TreeItem {
    mode: String,
    path: Path,
    // hex string referencing tree (directory) or blob (file)
    hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Tree {
    items: Vec<TreeItem>,
}

impl Tree {
    pub(crate) fn read(repo: &mut Repo, hash: &String) -> Result<Tree> {
        if let Some(tree) = repo.trees().read().get(hash) {
            log::debug!("tree: {tree}");
            Ok(tree)
        } else {
            Err(HashNotFound{hash})
        }
    }

    pub(crate) fn write(repo: &mut Repo, hash: &String, tree: &Tree) {
        repo.trees().write().insert(hash, tree);
    }
}