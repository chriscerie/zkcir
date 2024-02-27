use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};
use tempfile::{tempdir, TempDir};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

pub fn add_dir_to_zip<P: AsRef<Path>>(
    path: P,
    zip: &mut ZipWriter<File>,
    base_path: &str,
) -> io::Result<()> {
    let path = path.as_ref();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            let relative_path = if base_path.is_empty() {
                PathBuf::from(entry_path.file_name().unwrap())
            } else {
                PathBuf::from(base_path).join(entry_path.file_name().unwrap())
            };

            if entry_path.is_dir() {
                add_dir_to_zip(entry_path, zip, relative_path.to_str().unwrap())?;
            } else {
                zip.start_file(
                    relative_path.to_str().unwrap(),
                    FileOptions::default().compression_method(CompressionMethod::Stored),
                )?;
                let mut file = File::open(entry_path)?;
                io::copy(&mut file, zip)?;
            }
        }
    } else {
        zip.start_file(
            base_path,
            FileOptions::default().compression_method(CompressionMethod::Stored),
        )?;
        let mut file = File::open(path)?;
        io::copy(&mut file, zip)?;
    }

    Ok(())
}

// Returning TempDir is necessary so its ownership can transfer to the caller. Otherwise PathBuf would also be dropped
pub fn zip_path(unzipped_path: &Path) -> io::Result<(TempDir, PathBuf)> {
    let zipped_dir = tempdir()?;
    let zipped_dir_path = zipped_dir.path();

    let new_zip_path = zipped_dir_path.join("source.zip");
    let zip_file = std::fs::File::create(&new_zip_path)?;

    let mut zip = ZipWriter::new(zip_file);

    add_dir_to_zip(unzipped_path, &mut zip, "")?;

    zip.finish()?;

    Ok((zipped_dir, new_zip_path))
}
