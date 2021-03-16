#![allow(dead_code)]

use log::{debug, error};
use rusoto_core::*;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3, S3Client};
use std::{fs::File, future::Future, io::Read, path::Path, process::exit};
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
    let mut buf: Vec<u8> = vec![];
    
    let path = Path::new(file_path);
    match File::open(file_path) {
      Ok(mut file) => {
        file.read_to_end(&mut buf).unwrap();
      },
      Err(e) => {
        error!("Unable to read backup file. #Error: {}", e);
        exit(1);
      }
    }
    
    debug!("Buffer size: {}", buf.len());

    let request = PutObjectRequest {
      body: Some(buf.into()),
      bucket: self.bucket.to_owned(),
      key: format!("{}{}", self.key.to_owned(), path.file_name().unwrap().to_str().unwrap()),
      content_type: Some("application/x-tgz".to_string()),
      acl: Some("public-read".to_string()),
      ..Default::default()
    };

    match self.resolve(self.client.put_object(request)) {
      Ok(_) => debug!("Backup file uploaded to S3!"),
      Err(e) => { 
        error!("Failed to upload backup ({}) to S3!. #Error: {:?}", path.file_name().unwrap().to_str().unwrap(), e)
      }
    }
  }

  pub fn download(&self) {
    let request = GetObjectRequest {
      bucket: self.bucket.to_owned(),
      key: self.key.to_owned(),
      ..Default::default()
    };

    match self.resolve(self.client.get_object(request)) {
      Ok(output) => {
        debug!("Backup file downloaded from S3!");
        //TODO
        output.body;
      },
      Err(e) => error!("Failed to download backup from S3!. #Error: {:?}", e)
    }
  }

  fn resolve<F: Future>(&self, future: F) -> F::Output {
    self.runtime.block_on(future)
  }
}
