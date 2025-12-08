use std::fs::File;
use std::io;
use std::path::Path;
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::FileOptions;

pub fn compress_dir(dir: &str, zip_path: &str) -> crate::error::Result<()> {
    let base_path = Path::new(dir);

    if !base_path.exists() {
        return Err(crate::error::BotError::InternalError(format!("dir not exists: {}", dir)));
    }

    if !base_path.is_dir() {
        return Err(crate::error::BotError::InternalError(format!(
            "dir is not a directory: {}",
            dir
        )));
    }

    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 遍历目录树
    for entry in WalkDir::new(base_path) {
        let entry = entry?;
        let path = entry.path();

        let relative_path = path.strip_prefix(base_path).unwrap_or(path);

        if path.is_file() {
            zip.start_file(relative_path.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        } else if path != base_path {
            zip.add_directory(relative_path.to_string_lossy(), options)?;
        }
    }

    zip.finish()?;
    Ok(())
}
