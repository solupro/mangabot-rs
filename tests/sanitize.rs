use mangabot_rs::utils::fs::sanitize_filename;

#[test]
fn test_sanitize_filename_basic() {
    assert_eq!(sanitize_filename("a/b\\c*?<>|"), "a_b_c_____");
    assert_eq!(sanitize_filename("..hidden"), "hidden");
    assert_eq!(sanitize_filename(" 漫 画 标 题 "), "_ _ _ _");
}

#[test]
fn test_sanitize_filename_length_cap() {
    let long = "a".repeat(1000);
    let s = sanitize_filename(&long);
    assert!(s.len() <= 128);
}
