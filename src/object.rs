use crate::repo::Repo;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ObjectError {
    #[error("hash {hash} not found")]
    HashNotFound { hash: String },

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
    hash: String,
    content: [u8],
}

impl Blob {
    pub(crate) fn read(repo: &mut Repo, hash: &String) -> Result<Self, ObjectError> {
        if let Some(hex) = repo.blobs().read().get(hash) {
            log::debug!("hex: {hex}; hex_len: {}", hex.len());
            let b = Blob {
                hash,
                content: hex::decode(hex)?,
            }
            //let s = String::from_utf8(hex::decode(hex)?)?;
            log::debug!("as string: {}", String::from_utf8(b.content)?);
            Ok(b)
        } else {
            Err(HashNotFound{hash})
        }
    }

    pub(crate) fn write(repo: &mut Repo, blob: &Blob) {
        let hex = hex::encode(blob.content);
        log::debug!(
            "content: {:?}; content_len: {}; hex: {}; hex_len: {}",
            blob.content,
            blob.content.len(),
            hex,
            hex.len(),
        );

        repo.blobs().write().insert(blob.hash, hex);
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

    // TODO: gpgsig
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
