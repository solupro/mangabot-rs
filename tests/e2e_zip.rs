use mangabot_rs::utils::zip::compress_dir;
use std::fs;

#[test]
fn test_e2e_compress_dir_creates_zip() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("manga");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.txt"), b"hello").unwrap();
    fs::write(dir.join("b.txt"), b"world").unwrap();

    let zip_path = tmp.path().join("out.zip");
    compress_dir(dir.to_str().unwrap(), zip_path.to_str().unwrap()).unwrap();

    let meta = fs::metadata(&zip_path).unwrap();
    assert!(meta.len() > 0);
}
