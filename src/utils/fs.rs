use std::path::Path;

pub fn sanitize_filename(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        let ok = ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ' ');
        if ok {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim().trim_matches('.');
    let s = if trimmed.is_empty() { "file" } else { trimmed };
    s.chars().take(128).collect()
}

pub fn canonicalize_within(base: &Path, target: &Path) -> bool {
    if let (Ok(base), Ok(target)) = (base.canonicalize(), target.canonicalize()) {
        target.starts_with(&base)
    } else {
        false
    }
}
