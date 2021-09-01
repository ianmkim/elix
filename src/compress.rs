use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;

use zip::result::ZipError;
use zip::write::FileOptions;

use std::fs::File;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

const METHOD_STORED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);

#[cfg(any(
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib"
))]
const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);

#[cfg(not(any (
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib",
)))]
const METHOD_DEFLATED: Option<zip::CompressionMethod> = None;

#[cfg(feature="bzip2")]
const METHOD_BZIP2: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Bzip2);

#[cfg(not(feature="bzip2"))]
const METHOD_BZIP2: Option<zip::CompressionMethod> = None;

fn zip_dir_inner<T>(
    it: &mut dyn Iterator<Item=DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> where T: Write + Seek, {
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o775);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        if path.is_file() {
            println!("Adding file {:?} as {:?}... ", path, name);
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            println!("adding dir {:?} as {:?}...", path, name);
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn doit(
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

    zip_dir_inner(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;
    Ok(())
}

pub fn zip_dir(src: &str, dst: &str) {
    match doit(src, dst, zip::CompressionMethod::Stored) {
        Ok(_) => println!("Done: {} written to {}", src, dst),
        Err(e) => println!("Error: {:?}", e),
    }
}
