use crate::repo::Repo;
use diesel::prelude::*;
use sha1::{Digest, Sha1};
use simple_error::SimpleError;
// use simple_error::SimpleError;
use std::error::Error;

pub(crate) fn hash(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

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

    pub(crate) fn write(
        repo: &mut Repo,
        blob_hash: &String,
        blob_content: &Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::blobs::dsl::*;

        let db = repo.db();

        let new_blob = (hash.eq(blob_hash), content.eq(blob_content));

        match diesel::insert_into(blobs).values(&new_blob).execute(db) {
            Ok(n) => match n {
                1 => Ok(()),
                _ => Err(Box::new(SimpleError::new(format!(
                    "inserted {} rows unexpectedly",
                    n
                )))),
            },
            Err(e) => Err(Box::new(e)),
        }
    }
}
