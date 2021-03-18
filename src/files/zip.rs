use std::{fs::{self, File}, io::{self, Read, Seek, Write}, path::Path};

use walkdir::{DirEntry, WalkDir};
use zip::{result::ZipError, write::FileOptions};

const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);

pub fn do_zip(src_dir: &str, dst_file: &str) -> zip::result::ZipResult<()> {
  if !Path::new(src_dir).is_dir() {
    return Err(ZipError::FileNotFound);
  }

  let path = Path::new(dst_file);
  let file = File::create(&path).unwrap();

  let walkdir = WalkDir::new(src_dir.to_string());
  let it = walkdir.into_iter();

  zip_dir(
    &mut it.filter_map(|e| e.ok()),
    src_dir,
    file,
    METHOD_DEFLATED.unwrap(),
  )?;

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

pub fn do_unzip(src: &str, _dst: &str) {
  let fname = std::path::Path::new(src);
  let file = File::open(&fname).unwrap();

  let mut archive = zip::ZipArchive::new(file).unwrap();

  for i in 0..archive.len() {
    let mut file = archive.by_index(i).unwrap();
    let outpath = match file.enclosed_name() {
      Some(path) => path.to_owned(),
      None => continue,
    };

    {
      let comment = file.comment();
      if !comment.is_empty() {
        println!("File {} comment: {}", i, comment);
      }
    }

    if (&*file.name()).ends_with('/') {
      println!("File {} extracted to \"{}\"", i, outpath.display());
      fs::create_dir_all(&outpath).unwrap();
    } else {
      println!(
        "File {} extracted to \"{}\" ({} bytes)",
        i,
        outpath.display(),
        file.size()
      );
      if let Some(p) = outpath.parent() {
        if !p.exists() {
          fs::create_dir_all(&p).unwrap();
        }
      }
      let mut outfile = File::create(&outpath).unwrap();
      io::copy(&mut file, &mut outfile).unwrap();
    }

    // Get and Set permissions
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;

      if let Some(mode) = file.unix_mode() {
        fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
      }
    }
  }
}

#[cfg(test)]

mod test {
  use super::*;

  fn setup(paths: &[&Path]) {
    for path in paths {
      if path.exists() && path.is_file() {
        println!("# Removing zip file.");
        std::fs::remove_file(path).unwrap();
      }
      if path.exists() && path.is_dir() {
        println!("# Removing unzipped folder.");
        std::fs::remove_dir_all(path).unwrap();
      }
    }
  }

  #[test]
  fn test_zip() {
    let zip_path = Path::new("src_test.zip");
    setup(&[zip_path]);

    do_zip("src", "src_test.zip").unwrap();
    assert!(zip_path.exists() && zip_path.is_file());
  }

  #[test]
  fn test_unzip() {
    let zip_path = Path::new("src_test.zip");
    let path = Path::new("final_test");
    setup(&[zip_path, path]);

    do_zip("src", "src_test.zip").unwrap();
    assert!(zip_path.exists() && zip_path.is_file());

    do_unzip("src_test.zip", "final_test");
    assert!(path.exists() && path.is_dir());
  }

}