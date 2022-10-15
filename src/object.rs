use crate::repo::Repo;
use diesel::prelude::*;
use simple_error::SimpleError;
// use sha1::{Digest, Sha1};
// use simple_error::SimpleError;
use std::error::Error;

#[derive(Queryable)]
pub(crate) struct Blob {
    pub hash: String,
    pub content: Vec<u8>,
}

impl Blob {
    pub(crate) fn read(repo: &mut Repo, sha: &String) -> Result<String, Box<dyn Error>> {
        use crate::schema::blobs::dsl::*;

        let db = repo.db();

        let blob: Blob = blobs.find(sha).first(db)?;

        if blob.hash != *sha {
            return Err(Box::new(SimpleError::new(format!(
                "something very bad happened: hashes don't match: {} vs {}",
                sha, blob.hash
            ))));
        }

        Ok(String::from_utf8(blob.content)?)
    }
}

/*
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
*/
