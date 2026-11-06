// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Default drop-in directory for USB alias tables.
pub const DEFAULT_UALIAS_DIR: &str = "/etc/udev/ualias.d";

/// File within the drop-in directory used when appending a new pattern.
pub const APPEND_FILENAME: &str = "local";

#[derive(Error, Debug)]
pub enum UaliasError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid devices line: {0}")]
    InvalidLine(String),
    #[error("no matching alias")]
    NoMatch,
}

/// One row from an alias file: `VENDOR PRODUCT SERIAL NAME`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevicePattern {
    pub vendor: String,
    pub product: String,
    pub serial: String,
    pub name: String,
}

impl DevicePattern {
    pub fn new(
        vendor: impl Into<String>,
        product: impl Into<String>,
        serial: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            vendor: vendor.into(),
            product: product.into(),
            serial: serial.into(),
            name: name.into(),
        }
    }

    /// Format as a devices-file line (space-separated fields).
    pub fn to_line(&self) -> String {
        format!(
            "{} {} {} {}",
            self.vendor, self.product, self.serial, self.name
        )
    }

    /// True when this pattern matches the query. A pattern field of `*` matches
    /// any query value (including absent). A concrete pattern field matches only
    /// when the query provides the same value (ASCII case-insensitive).
    pub fn matches(
        &self,
        vendor: Option<&str>,
        product: Option<&str>,
        serial: Option<&str>,
    ) -> bool {
        field_matches(&self.vendor, vendor)
            && field_matches(&self.product, product)
            && field_matches(&self.serial, serial)
    }
}

fn field_matches(pattern: &str, query: Option<&str>) -> bool {
    if pattern == "*" {
        return true;
    }
    match query {
        Some(q) => pattern.eq_ignore_ascii_case(q),
        None => false,
    }
}

/// Uppercase the first Unicode scalar of `name`, leaving the rest unchanged.
pub fn ucfirst(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut s = c.to_uppercase().collect::<String>();
            s.push_str(chars.as_str());
            s
        }
    }
}

/// Parse devices-file contents. Blank lines and `#` comments are ignored.
pub fn parse_devices(text: &str) -> Result<Vec<DevicePattern>, UaliasError> {
    let mut out = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 4 {
            return Err(UaliasError::InvalidLine(format!(
                "line {}: expected 4 fields, got {}",
                idx + 1,
                parts.len()
            )));
        }
        out.push(DevicePattern::new(parts[0], parts[1], parts[2], parts[3]));
    }
    Ok(out)
}

/// Whether `name` should be loaded from a `*.d` drop-in directory.
pub fn is_dropin_filename(name: &str) -> bool {
    if name.is_empty() || name.starts_with('.') {
        return false;
    }
    if name.ends_with('~') {
        return false;
    }
    // Packaging leftovers / editor backups
    for needle in [".dpkg-", ".rpm", ".bak", ".old", ".orig", ".swp"] {
        if name.contains(needle) {
            return false;
        }
    }
    true
}

/// Sorted list of regular drop-in files under `dir`.
pub fn list_dropin_files(dir: &Path) -> Result<Vec<PathBuf>, UaliasError> {
    let mut files = Vec::new();
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(files),
        Err(e) => return Err(e.into()),
    };
    for entry in rd {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !is_dropin_filename(name) {
            continue;
        }
        files.push(entry.path());
    }
    files.sort();
    Ok(files)
}

/// Load and parse a single alias file.
pub fn load_devices_file(path: &Path) -> Result<Vec<DevicePattern>, UaliasError> {
    let file = File::open(path)?;
    let mut text = String::new();
    for line in BufReader::new(file).lines() {
        text.push_str(&line?);
        text.push('\n');
    }
    parse_devices(&text).map_err(|e| match e {
        UaliasError::InvalidLine(msg) => {
            UaliasError::InvalidLine(format!("{}: {msg}", path.display()))
        }
        other => other,
    })
}

/// Load patterns from every drop-in file under `dir`, in sorted filename order.
/// Missing directory yields an empty pattern list (no error).
pub fn load_devices_dir(dir: &Path) -> Result<Vec<DevicePattern>, UaliasError> {
    let mut out = Vec::new();
    for path in list_dropin_files(dir)? {
        out.extend(load_devices_file(&path)?);
    }
    Ok(out)
}

/// First matching pattern in order, or [`UaliasError::NoMatch`].
pub fn lookup_name(
    patterns: &[DevicePattern],
    vendor: Option<&str>,
    product: Option<&str>,
    serial: Option<&str>,
) -> Result<String, UaliasError> {
    for p in patterns {
        if p.matches(vendor, product, serial) {
            return Ok(p.name.clone());
        }
    }
    Err(UaliasError::NoMatch)
}

/// Path of the file used for append mode inside `dir`.
pub fn append_file_path(dir: &Path) -> PathBuf {
    dir.join(APPEND_FILENAME)
}

/// Append one pattern line to `dir/local`, creating the directory if needed.
pub fn append_pattern(dir: &Path, pattern: &DevicePattern) -> Result<(), UaliasError> {
    fs::create_dir_all(dir)?;
    let path = append_file_path(dir);
    let needs_nl = match fs::metadata(&path) {
        Ok(meta) if meta.len() > 0 => {
            let data = fs::read(&path)?;
            !matches!(data.last(), Some(b'\n'))
        }
        _ => false,
    };
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    if needs_nl {
        file.write_all(b"\n")?;
    }
    writeln!(file, "{}", pattern.to_line())?;
    Ok(())
}

/// Build a pattern for append: missing vendor/product/serial become `*`.
pub fn pattern_for_append(
    vendor: Option<&str>,
    product: Option<&str>,
    serial: Option<&str>,
    name: &str,
) -> DevicePattern {
    DevicePattern::new(
        vendor.unwrap_or("*"),
        product.unwrap_or("*"),
        serial.unwrap_or("*"),
        name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SAMPLE: &str = "\
#VENDOR PRODUCT SERIAL NAME
* * 44:1B:F6:D7:92:5C s3cam
* * 10:20:BA:3D:30:28 s3tlcd3.49in
* * 3C:DC:75:70:15:2C s3tlcd2.0in
* * 44:1B:F6:D2:22:1F s3cam
303a 1001 * s3g
";

    fn tmp_dir(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "ualias-{}-{}",
            tag,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn parse_sample() {
        let pats = parse_devices(SAMPLE).unwrap();
        assert_eq!(pats.len(), 5);
        assert_eq!(pats[0].name, "s3cam");
        assert_eq!(pats[4].vendor, "303a");
    }

    #[test]
    fn serial_exact_match() {
        let pats = parse_devices(SAMPLE).unwrap();
        let name = lookup_name(&pats, None, None, Some("44:1B:F6:D7:92:5C")).unwrap();
        assert_eq!(name, "s3cam");
        let name = lookup_name(&pats, None, None, Some("44:1b:f6:d7:92:5c")).unwrap();
        assert_eq!(name, "s3cam");
    }

    #[test]
    fn vendor_product_fallback() {
        let pats = parse_devices(SAMPLE).unwrap();
        let name = lookup_name(&pats, Some("303a"), Some("1001"), Some("DEADBEEF")).unwrap();
        assert_eq!(name, "s3g");
        let name = lookup_name(&pats, Some("303A"), Some("1001"), Some("xx")).unwrap();
        assert_eq!(name, "s3g");
    }

    #[test]
    fn vendor_product_without_serial_skips_serial_rows() {
        let pats = parse_devices(SAMPLE).unwrap();
        let name = lookup_name(&pats, Some("303a"), Some("1001"), None).unwrap();
        assert_eq!(name, "s3g");
    }

    #[test]
    fn no_match() {
        let pats = parse_devices(SAMPLE).unwrap();
        let err = lookup_name(&pats, Some("1234"), Some("5678"), Some("none")).unwrap_err();
        assert!(matches!(err, UaliasError::NoMatch));
    }

    #[test]
    fn ucfirst_basic() {
        assert_eq!(ucfirst("s3cam"), "S3cam");
        assert_eq!(ucfirst(""), "");
        assert_eq!(ucfirst("已有"), "已有");
    }

    #[test]
    fn dropin_filename_filter() {
        assert!(is_dropin_filename("10-serials"));
        assert!(is_dropin_filename("local"));
        assert!(!is_dropin_filename(".hidden"));
        assert!(!is_dropin_filename("local~"));
        assert!(!is_dropin_filename("foo.dpkg-old"));
        assert!(!is_dropin_filename("foo.bak"));
    }

    #[test]
    fn load_dir_sorted_and_merge() {
        let dir = tmp_dir("merge");
        fs::write(dir.join("20-fallback"), "303a 1001 * s3g\n").unwrap();
        fs::write(
            dir.join("10-serials"),
            "* * 44:1B:F6:D7:92:5C s3cam\n* * AA:BB:CC:DD:EE:FF other\n",
        )
        .unwrap();
        fs::write(dir.join(".skip"), "* * 00:00:00:00:00:00 hidden\n").unwrap();
        fs::write(dir.join("local~"), "* * 11:11:11:11:11:11 backup\n").unwrap();

        let pats = load_devices_dir(&dir).unwrap();
        assert_eq!(pats.len(), 3);
        assert_eq!(pats[0].name, "s3cam");
        assert_eq!(pats[1].name, "other");
        assert_eq!(pats[2].name, "s3g");

        let name = lookup_name(&pats, Some("303a"), Some("1001"), Some("DEAD")).unwrap();
        assert_eq!(name, "s3g");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_dir_is_empty() {
        let dir = tmp_dir("missing-parent").join("nope");
        let pats = load_devices_dir(&dir).unwrap();
        assert!(pats.is_empty());
        let _ = fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn append_and_reload() {
        let dir = tmp_dir("append");
        let pat = pattern_for_append(None, None, Some("AA:BB:CC:DD:EE:FF"), "newt");
        append_pattern(&dir, &pat).unwrap();
        append_pattern(
            &dir,
            &pattern_for_append(Some("303a"), Some("1001"), None, "s3g"),
        )
        .unwrap();
        let pats = load_devices_dir(&dir).unwrap();
        assert_eq!(pats.len(), 2);
        assert_eq!(pats[0].to_line(), "* * AA:BB:CC:DD:EE:FF newt");
        assert_eq!(pats[1].to_line(), "303a 1001 * s3g");
        assert!(append_file_path(&dir).ends_with(APPEND_FILENAME));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_line() {
        let err = parse_devices("* * onlythree\n").unwrap_err();
        assert!(matches!(err, UaliasError::InvalidLine(_)));
    }
}
