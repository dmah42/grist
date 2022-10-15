use crate::repo::Repo;
use sha1::{Digest, Sha1};
use simple_error::SimpleError;
use std::error::Error;

pub(crate) trait Object {
    fn new(repo: &Repo, raw: &[u8]) -> Self
    where
        Self: Sized;
}

impl dyn Object {
    pub(crate) fn read(repo: &Repo, sha: Sha1) -> Result<Box<dyn Object>, Box<dyn Error>> {
        let digest = hex::encode(sha.finalize());
        let dir = &digest[0..2];
        let file = &digest[2..];
        let path = repo.gristdir().join("objects").join(dir).join(file);

        let data = std::fs::read(path)?;

        println!("{:?}", data);

        let maybe_x = data.iter().position(|&x| x == b' ');
        if maybe_x.is_none() {
            return Err(Box::new(SimpleError::new("malformed object")));
        }

        let x = maybe_x.unwrap();

        let t = String::from_utf8(data[0..x].to_vec())?;

        log::info!("creating object type {}", t);

        let raw = &data[x..];

        match t.as_str() {
            "commit" => Ok(Box::new(Commit::new(repo, raw))),
            "tree" => Ok(Box::new(Tree::new(repo, raw))),
            "tag" => Ok(Box::new(Tag::new(repo, raw))),
            "blob" => Ok(Box::new(Blob::new(repo, raw))),
            s => Err(Box::new(SimpleError::new(format!(
                "unknown object type {}",
                s
            )))),
        }
    }
}

pub(crate) struct Commit {}

impl Object for Commit {
    fn new(repo: &Repo, raw: &[u8]) -> Self {
        Commit {}
    }
}

pub(crate) struct Tree {}

impl Object for Tree {
    fn new(repo: &Repo, raw: &[u8]) -> Self {
        Tree {}
    }
}

pub(crate) struct Tag {}

impl Object for Tag {
    fn new(repo: &Repo, raw: &[u8]) -> Self {
        Tag {}
    }
}

pub(crate) struct Blob {}

impl Object for Blob {
    fn new(repo: &Repo, raw: &[u8]) -> Self {
        Blob {}
    }
}
