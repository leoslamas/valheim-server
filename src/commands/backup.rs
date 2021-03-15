use clap::ArgMatches;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{debug, error};
use std::{process::exit};
use std::{fs::File};
use path_abs;

use crate::aws::s3::{S3Sync};
use crate::utils::fetch_env;

pub fn invoke(args: &ArgMatches) {
  let input = args.value_of("INPUT_DIR").unwrap();
  let output = args.value_of("OUTPUT_FILE").unwrap();
  let backup_to_s3 = fetch_env("BACKUP_TO_S3", "0", false).eq("1");

  let bucket = fetch_env("S3_BUCKET", "amnesicbit", false);
  let key = fetch_env("S3_KEY", "valheim/backups/", false);

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

  if backup_to_s3 {
    match File::open(output) {
        Ok(file) => {
          S3Sync::new(bucket, key).upload(file);
          debug!("Backup uploaded to S3 successfully!");
        },
        Err(_) => {
          error!("Failed to upload {} to backup file", input);
          exit(1)
        }
    }
  }

}
