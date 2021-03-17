use zip::{write::FileOptions, result::ZipError};
use clap::ArgMatches;
use log::{debug, error};
use std::{io::{prelude::*, Seek, Write}, fs::File, iter::Iterator, path::Path, process::exit};
use walkdir::{DirEntry, WalkDir};

use crate::aws::s3::{S3Sync};
use crate::utils::fetch_env;

const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);

pub fn invoke(args: &ArgMatches) {
  let input = args.value_of("INPUT_DIR").unwrap();
  let output = args.value_of("OUTPUT_FILE").unwrap();
  let backup_to_s3 = fetch_env("BACKUP_TO_S3", "0", false).eq("1");

  let bucket = fetch_env("S3_BUCKET", "amnesicbit", false);
  let key = fetch_env("S3_KEY", "valheim/backups/", false);

  debug!("Creating archive of {}", input);
  debug!("Output set to {}", output);
  
  match do_zip(input, output, METHOD_DEFLATED.unwrap()) {
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
          S3Sync::new(bucket, key).upload(output);
          debug!("Backup uploaded to S3 successfully!");
        },
        Err(e) => {
          error!("An error occurred while uploading backup file. \n#Error: {:?}", e);
          exit(1)
        }
    }
  }

}

fn do_zip(
  src_dir: &str,
  dst_file: &str,
  method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {
  if !Path::new(src_dir).is_dir() {
      return Err(ZipError::FileNotFound);
  }

  let path = Path::new(dst_file);
  let file = File::create(&path).unwrap();

  let walkdir = WalkDir::new(src_dir.to_string());
  let it = walkdir.into_iter();

  zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

  Ok(())
}

fn zip_dir<T>(
  it: &mut dyn Iterator<Item = DirEntry>,
  prefix: &str,
  writer: T,
  method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
  T: Write + Seek,
{
  let mut zip = zip::ZipWriter::new(writer);
  let options = FileOptions::default()
      .compression_method(method)
      .unix_permissions(0o755);

  let mut buffer = Vec::new();
  for entry in it {
      let path = entry.path();
      let name = path.strip_prefix(Path::new(prefix)).unwrap();

      // Write file or directory explicitly
      // Some unzip tools unzip files with directory paths correctly, some do not!
      if path.is_file() {
          println!("adding file {:?} as {:?} ...", path, name);
          #[allow(deprecated)]
          zip.start_file_from_path(name, options)?;
          let mut f = File::open(path)?;

          f.read_to_end(&mut buffer)?;
          zip.write_all(&*buffer)?;
          buffer.clear();
      } else if name.as_os_str().len() != 0 {
          // Only if not root! Avoids path spec / warning
          // and mapname conversion failed error on unzip
          println!("adding dir {:?} as {:?} ...", path, name);
          #[allow(deprecated)]
          zip.add_directory_from_path(name, options)?;
      }
  }
  zip.finish()?;
  Result::Ok(())
}