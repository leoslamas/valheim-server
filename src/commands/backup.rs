use clap::ArgMatches;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{debug, error};
use std::fs::File;
use std::process::exit;
use crate::utils::fetch_env;

use rusoto_core::{self, Region};
use rusoto_s3::{self, PutObjectRequest, S3Client, StreamingBody};

pub fn invoke(args: &ArgMatches) {
  let input = args.value_of("INPUT_DIR").unwrap();
  let output = args.value_of("OUTPUT_FILE").unwrap();
  
  let backup_to_s3 = fetch_env("BACKUP_TO_S3", "0", false).eq("1");
  let s3_bucket = fetch_env("S3_BUCKET", "amnesicbit", false);
  let s3_key = fetch_env("S3_KEY", "backups/", false);

  debug!("Creating archive of {}", input);
  debug!("Output set to {}", output);
  let tar_gz = match File::create(output) {
    Ok(file) => file,
    Err(_) => {
      error!("Failed to create backup file at {}", output);
      exit(1)
    }
  };
  let enc = GzEncoder::new(tar_gz, Compression::default());
  let mut tar = tar::Builder::new(enc);
  match tar.append_dir_all("saves", input) {
    Ok(_) => debug!("Successfully created backup zip at {}", output),
    Err(_) => {
      error!("Failed to add {} to backup file", input);
      exit(1)
    }
  };

  let body = StreamingBody::new();

  let client = S3Client::new(Region::SaEast1);
  let request = PutObjectRequest {
    body: tar_gz,
    bucket: s3_bucket,
    key: s3_key,
    content_type: Some("application/zip".to_string()),
    acl: Some("public-read".to_string()),
    ..Default::default()
  };
  
}

async fn do_upload() -> Result<(), Box<dyn std::error::Error>> {
    let s3_client = S3Client::new(Region::UsEast1);
    let mut file = tokio::fs::File::open("text.txt").await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let result = s3_client.put_object(PutObjectRequest {
        bucket: String::from("bucket-name"),
        key: "text.txt".to_string(),
        body: Some(StreamingBody::from(buffer)),
        ..Default::default()
    }).await?;
    // do thing with result
    Ok(())
}