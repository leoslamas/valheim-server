#![allow(dead_code)]

use log::{debug, error};
use rusoto_core::{ByteStream, Region};
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3, S3Client};
use std::{fs::File, future::Future, io::Read};
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
      .build()
      .unwrap();

    S3Sync {
      bucket,
      key,
      client: cl,
      runtime: rt,
    }
  }

  pub fn upload(&self, mut file: File) {
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).unwrap();
    let byte_stream = <ByteStream as From<Vec<u8>>>::from(buf);

    let request = PutObjectRequest {
      body: Some(byte_stream),
      bucket: self.bucket.clone(),
      key: self.key.clone(),
      content_type: Some("application/zip".to_string()),
      acl: Some("public-read".to_string()),
      ..Default::default()
    };

    match self.resolve(self.client.put_object(request)) {
      Ok(_) => debug!("Backup file uploaded to S3!"),
      Err(_) => error!("Failed to upload backup ({:?}) to S3!", file)
    }
  }

  pub fn download(&self) {
    let request = GetObjectRequest {
      bucket: self.bucket.clone(),
      key: self.key.clone(),
      ..Default::default()
    };

    match self.resolve(self.client.get_object(request)) {
      Ok(output) => {
        debug!("Backup file downloaded from S3!");
        //TODO
        output.body;
      },
      Err(_) => error!("Failed to download backup from S3!")
    }
  }

  fn resolve<F: Future>(&self, future: F) -> F::Output {
    self.runtime.block_on(future)
  }
}
