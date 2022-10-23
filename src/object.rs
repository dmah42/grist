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

        let select_stmt = format!(r#"SELECT hash, content FROM Blobs WHERE hash = "{}""#, sha);

        log::debug!("object read: {}", select_stmt);

        let payloads = db.execute(select_stmt)?;

        log::debug!("{} payloads found", payloads.len());
        if payloads.len() != 1 {
            return Err(Box::new(SimpleError::new(format!(
                "expected 1 payload, got {}",
                payloads.len()
            ))));
        }

        let row = match &payloads[0] {
            Payload::Select { labels: _, rows } => {
                log::debug!("{} rows found", rows.len());
                if rows.len() != 1 {
                    return Err(Box::new(SimpleError::new(format!(
                        "expected 1 row, got {}",
                        rows.len()
                    ))));
                }
                log::debug!("row: {:?}", &rows[0]);
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
                    return Err(Box::new(SimpleError::new(format!(
                        "unexpected value type for hash: {:?}",
                        v
                    ))))
                }
            },
        };
        log::debug!("hash: {}; sha: {}", hash, sha);
        if hash.ne(sha) {
            return Err(Box::new(SimpleError::new(format!(
                "something very bad happened: hashes don't match: {} vs {}",
                sha, hash
            ))));
        }

        let hex = match row.get_value_by_index(1) {
            None => return Err(Box::new(SimpleError::new("no content found"))),
            Some(v) => match v {
                Value::Str(s) => s,
                _ => {
                    return Err(Box::new(SimpleError::new(format!(
                        "unexpected value type for content: {:?}",
                        v
                    ))));
                }
            },
        };
        log::debug!("hex: {:?}; hex_len: {}", hex, hex.len());
        let decoded = hex::decode(hex)?;
        log::debug!("decoded: {:?}; decoded_len: {}", decoded, decoded.len());
        let s = String::from_utf8(decoded)?;
        log::debug!("as string: {:?}", s);
        Ok(s)
    }

    pub(crate) fn write(
        repo: &mut Repo,
        hash: &String,
        content: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let db = repo.db();

        let hex = hex::encode(content);
        log::debug!(
            "content: {:?}; content_len: {}; hex: {}; hex_len: {}",
            content,
            content.len(),
            hex,
            hex.len(),
        );

        let insert_stmt = format!(r#"INSERT INTO Blobs VALUES ("{}", "{}")"#, hash, hex);

        log::debug!("insert statement: {}", insert_stmt);

        match db.execute(insert_stmt) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}
