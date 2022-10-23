use crate::repo::Repo;
use gluesql::prelude::*;
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
        let db = repo.db();

        let payloads = db.execute(format!(
            "SELECT hash, content FROM Blobs WHERE hash = {}",
            sha
        ))?;

        if payloads.len() != 1 {
            return Err(Box::new(SimpleError::new(format!(
                "expected 1 payload, got {}",
                payloads.len()
            ))));
        }

        let row = match &payloads[0] {
            Payload::Select { labels: _, rows } => {
                if rows.len() != 1 {
                    return Err(Box::new(SimpleError::new(format!(
                        "expected 1 row, got {}",
                        rows.len()
                    ))));
                }
                &rows[0]
            }
            _ => {
                return Err(Box::new(SimpleError::new(format!(
                    "unexpected payload type {:?}",
                    payloads[0]
                ))));
            }
        };

        let hash = match row.get_value_by_index(0) {
            None => return Err(Box::new(SimpleError::new("no hash found"))),
            Some(v) => match v {
                Value::Str(s) => s,
                _ => {
                    return Err(Box::new(SimpleError::new("unexpected value type")));
                }
            },
        };
        if hash.ne(sha) {
            return Err(Box::new(SimpleError::new(format!(
                "something very bad happened: hashes don't match: {} vs {}",
                sha, hash
            ))));
        }

        let content = match row.get_value_by_index(1) {
            None => return Err(Box::new(SimpleError::new("no content found"))),
            Some(v) => match v {
                Value::Bytea(bytes) => bytes,
                _ => {
                    return Err(Box::new(SimpleError::new("unexpected value type")));
                }
            },
        };
        Ok(String::from_utf8(content.to_vec())?)
    }

    pub(crate) fn write(
        repo: &mut Repo,
        hash: &String,
        content: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let db = repo.db();

        match db.execute(format!(
            "INSERT INTO Blobs VALUES ({}, {})",
            hash,
            String::from_utf8(content.to_vec())?
        )) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}
