// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use thiserror::Error;

/// Default USB alias table path.
pub const DEFAULT_DEVICES_PATH: &str = "/etc/udev/devices";

#[derive(Error, Debug)]
pub enum UaliasError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid devices line: {0}")]
    InvalidLine(String),
    #[error("no matching alias")]
    NoMatch,
}

/// One row from the devices file: `VENDOR PRODUCT SERIAL NAME`.
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

/// Load and parse the devices file at `path`.
pub fn load_devices(path: &Path) -> Result<Vec<DevicePattern>, UaliasError> {
    let file = File::open(path)?;
    let mut text = String::new();
    for line in BufReader::new(file).lines() {
        text.push_str(&line?);
        text.push('\n');
    }
    parse_devices(&text)
}

/// First matching pattern in file order, or [`UaliasError::NoMatch`].
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

/// Append one pattern line to `path`, creating the file if needed.
pub fn append_pattern(path: &Path, pattern: &DevicePattern) -> Result<(), UaliasError> {
    let needs_nl = match std::fs::metadata(path) {
        Ok(meta) if meta.len() > 0 => {
            let data = std::fs::read(path)?;
            !matches!(data.last(), Some(b'\n'))
        }
        _ => false,
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
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

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "ualias-{}-{}",
            tag,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
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
        // Case-insensitive vendor/product
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
    fn append_and_reload() {
        let path = tmp_path("append");
        let _ = std::fs::remove_file(&path);
        let pat = pattern_for_append(None, None, Some("AA:BB:CC:DD:EE:FF"), "newt");
        append_pattern(&path, &pat).unwrap();
        append_pattern(
            &path,
            &pattern_for_append(Some("303a"), Some("1001"), None, "s3g"),
        )
        .unwrap();
        let pats = load_devices(&path).unwrap();
        assert_eq!(pats.len(), 2);
        assert_eq!(pats[0].to_line(), "* * AA:BB:CC:DD:EE:FF newt");
        assert_eq!(pats[1].to_line(), "303a 1001 * s3g");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn invalid_line() {
        let err = parse_devices("* * onlythree\n").unwrap_err();
        assert!(matches!(err, UaliasError::InvalidLine(_)));
    }
}
