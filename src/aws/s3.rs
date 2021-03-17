#![allow(dead_code)]

use log::{debug, error};
use rusoto_core::*;
use rusoto_s3::{PutObjectRequest, S3, S3Client, StreamingBody};
use std::{fs::{File, read}, future::Future, path::Path};
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
          error!("Unable to upload backup file!. #Error: {}", e);
        }
    }
  }

  async fn do_upload(&self,file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    debug!("File path: {}", file_path);
    let file = File::open(file_path).unwrap();
    let path = Path::new(file_path);
    let size = file.metadata().unwrap().len();

    debug!("File size: {}", size);
    let reader = read(path).unwrap();

    self.client.put_object(PutObjectRequest {
        bucket: self.bucket.to_owned(),
        key: format!("{}{}", self.key.to_owned(), path.file_name().unwrap().to_str().unwrap()),
        content_type: Some("application/zip".to_string()),
        body: Some(StreamingBody::from(reader)),
        ..PutObjectRequest::default()
    }).await?;
    
    Ok(())
  }

  fn resolve<F: Future>(&self, future: F) -> F::Output {
    self.runtime.block_on(future)
  }
}
