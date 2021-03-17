#![allow(dead_code)]

use log::{debug, error};
use rusoto_core::*;
use rusoto_s3::{CopyObjectRequest, PutObjectRequest, S3, S3Client, StreamingBody};
use std::{fs::{read, File}, future::Future, path::Path, process::exit};
use tokio::runtime;

pub struct S3Sync {
  bucket: String,
  key: String,
  client: S3Client,
  runtime: runtime::Runtime,
}

impl S3Sync {
  pub fn new(bucket: String, key: String) -> Self {
    let cl = S3Client::new(Region::SaEast1);
    let rt = runtime::Builder::new_current_thread()
      .enable_io()
      .build()
      .unwrap();

    S3Sync {
      bucket,
      key,
      client: cl,
      runtime: rt,
    }
  }

  pub fn upload(&self, file_path: &str) {
    debug!("Starting upload of {}", file_path);

    match self.resolve(self.do_upload(file_path)) {
        Ok(_) => {
          debug!("Backup successfully uploaded!");
        }
        Err(e) => {
          error!("Error setting up backup file!. #Error: {}", e);
          exit(1)
        }
    }
  }

  async fn do_upload(&self,file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    debug!("File path: {}", file_path);
    let file = File::open(file_path).unwrap();
    let path = Path::new(file_path);
    let size = file.metadata().unwrap().len();
    let put_key = format!(
      "{}{}",
      self.key.to_owned(),
      path.file_name().unwrap().to_str().unwrap()
    );
    let copy_key = format!("{}{}", self.key.to_owned(), "backup.zip");
    let content_type = Some("application/zip".to_string());
    let acl = Some("public-read".to_string());

    debug!("File size: {}", size);
    let reader = read(path).unwrap();

    debug!("Put key: {}{}", self.bucket.to_owned(), put_key.to_owned());
    debug!("Copy key: {}{}", self.bucket.to_owned(), copy_key.to_owned());

    self
      .client
      .put_object(PutObjectRequest {
        bucket: self.bucket.to_owned(),
        key: put_key.to_owned(),
        body: Some(StreamingBody::from(reader)),
        content_type: content_type.to_owned(),
        acl: acl.to_owned(),
        ..PutObjectRequest::default()
      })
      .await.or_else(|e| {
        error!("Unable to put object.");
        Err(e)
      })?;
          

    self
      .client
      .copy_object(CopyObjectRequest {
        bucket: self.bucket.to_owned(),
        key: copy_key.to_owned(),
        copy_source: put_key.to_owned(),
        content_type: content_type.to_owned(),
        acl: acl.to_owned(),
        ..CopyObjectRequest::default()
      })
      .await.or_else(|e| {
        error!("Unable to copy object.");
        Err(e)
      })?;

    Ok(())
  }

  async fn do_download(&self, _dst_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  fn resolve<F: Future>(&self, future: F) -> F::Output {
    self.runtime.block_on(future)
  }
}
