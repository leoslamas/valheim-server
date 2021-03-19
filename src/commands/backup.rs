use clap::ArgMatches;
use log::{debug, error};
use std::{fs::File, process::exit};

use crate::aws::s3::S3Sync;
use crate::utils::fetch_env;
use crate::files::zip;

pub fn invoke(args: &ArgMatches) {
  let input = args.value_of("INPUT_DIR").unwrap();
  let output = args.value_of("OUTPUT_FILE").unwrap();
  let backup_to_s3 = fetch_env("BACKUP_TO_S3", "0", false).eq("1");

  debug!("Creating archive of {}", input);
  debug!("Output set to {}", output);
  
  match zip::do_zip(input, output) {
      Ok(_) => {
        debug!("Backup file created!");
      }
      Err(e) => {
        error!("Unable to create backup file!. \n#Error: {}", e);
      }
  };

  if backup_to_s3 {
    match File::open(output) {
        Ok(_) => {
          S3Sync::new_default().upload(output);
          println!("Backup uploaded to S3 successfully!");
        },
        Err(e) => {
          error!("An error occurred while uploading backup file. \n#Error: {:?}", e);
          exit(1)
        }
    }
  } else {
    println!("Backup to S3 disabled.");
  }

}
