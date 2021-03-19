#![allow(dead_code)]

use log::{debug, error};
use rusoto_core::*;
use rusoto_s3::{
  GetObjectRequest, PutObjectRequest, S3Client, StreamingBody, S3,
};
use std::{fs::{read, File}, future::Future, io::Write, path::Path, process::exit};
use tokio::{io::AsyncReadExt, runtime};

use crate::utils::fetch_env;

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

  pub fn new_default() -> Self {
    let bucket = fetch_env("S3_BUCKET", "amnesicbit", false);
    let key = fetch_env("S3_KEY", "valheim/backups/", false);
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

  async fn do_upload(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let put_key = format!(
      "{}{}",
      self.key.to_owned(),
      path.file_name().unwrap().to_str().unwrap()
    );
    let copy_key = format!("{}{}", self.key.to_owned(), "backup.zip");
    let content_type = Some("application/zip".to_string());
    let acl = Some("public-read-write".to_string());

    let reader = read(path).unwrap();

    self
      .client
      .put_object(PutObjectRequest {
        bucket: self.bucket.to_owned(),
        key: put_key.to_owned(),
        body: Some(StreamingBody::from(reader.to_owned())),
        content_type: content_type.to_owned(),
        acl: acl.to_owned(),
        ..PutObjectRequest::default()
      })
      .await
      .or_else(|e| {
        error!("Unable to put object.");
        Err(e)
      })?;

      //TODO make this a copy-object
      self
      .client
      .put_object(PutObjectRequest {
        bucket: self.bucket.to_owned(),
        key: copy_key.to_owned(),
        body: Some(StreamingBody::from(reader.to_owned())),
        content_type: content_type.to_owned(),
        acl: acl.to_owned(),
        ..PutObjectRequest::default()
      })
      .await
      .or_else(|e| {
        error!("Unable to copy object.");
        Err(e)
      })?;

    Ok(())
  }

  pub fn download_backup(&self, dst_path: &str) {
    match self.resolve(self.do_download(dst_path)) {
      Ok(_) => {
        debug!("Backup successfully downloaded!");
      }
      Err(e) => {
        error!("Error downloading backup file!. #Error: {}", e);
        exit(1)
      }
    }
  }

  async fn do_download(
    &self,
    dst_path: &str,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("{}{}", self.key.to_owned(), "backup.zip");

    match self
      .client
      .get_object(GetObjectRequest {
        bucket: self.bucket.to_owned(),
        key: key,
        ..GetObjectRequest::default()
      })
      .await {
          Ok(out) => {
            debug!("Content length: {}", out.content_length.unwrap());
            let stream = out.body.unwrap();
            let mut buf = Vec::with_capacity(out.content_length.unwrap() as usize);
            stream.into_async_read().read_to_end(&mut buf).await?;

            let mut backup_file = File::create(dst_path)?;
            backup_file.write_all(&buf)?;
          }

          Err(e) => {
            error!("Unable to download backup file.");
            return Err(Box::new(e));
          }
      }

    Ok(())
  }

  fn resolve<F: Future>(&self, future: F) -> F::Output {
    self.runtime.block_on(future)
  }
}
