use crate::utils::fetch_env;
use clap::ArgMatches;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{debug, error};
use std::process::exit;
use std::{fs::File, io::Read};

use rusoto_core::{self, ByteStream, Region};
use rusoto_s3::{self, PutObjectRequest, S3Client, S3};
use tokio;

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

  if !backup_to_s3 {
    return;
  }
  if let Ok(mut file) = File::open(output) {
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).unwrap();
    let byte_stream = ByteStream::from(buf);

    let client = S3Client::new(Region::SaEast1);
    let request = PutObjectRequest {
      body: Some(byte_stream),
      bucket: s3_bucket,
      key: s3_key,
      content_type: Some("application/zip".to_string()),
      acl: Some("public-read".to_string()),
      ..Default::default()
    };

    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();

    match rt.block_on(client.put_object(request)) {
        Ok(_) => debug!("Backup file uploaded to S3!"),
        Err(_) => error!("Failed to upload backup ({}) to S3", output)
    };
    
  } else {
    error!("Failed to open backup file ({})", output)
  }
}
