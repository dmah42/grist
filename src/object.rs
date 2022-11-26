use crate::repo::Repo;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use simple_error::SimpleError;
use std::error::Error;

pub(crate) fn hash(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub(crate) struct Blob {}

impl Blob {
    pub(crate) fn read(repo: &mut Repo, sha: &String) -> Result<String, Box<dyn Error>> {
        if let Some(hex) = repo.blobs().read().get(sha) {
            log::debug!("hex: {hex}; hex_len: {}", hex.len());
            let s = String::from_utf8(hex::decode(hex)?)?;
            log::debug!("as string: {s}");
            Ok(s)
        } else {
            Err(Box::new(SimpleError::new(format!("hash {sha} not found"))))
        }
    }

    pub(crate) fn write(repo: &mut Repo, hash: &String, content: &[u8]) {
        let hex = hex::encode(content);
        log::debug!(
            "content: {:?}; content_len: {}; hex: {}; hex_len: {}",
            content,
            content.len(),
            hex,
            hex.len(),
        );

        repo.blobs().write().insert(hash.to_string(), hex);
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

    /// hex string
    parent: String,

    author: Author,

    committer: Author,

    // TODO: gpgsig
    comment: String,
}

impl Commit {}
