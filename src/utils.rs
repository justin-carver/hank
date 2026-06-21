use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Boxed, thread-safe error so these helpers compose with `?` across both
/// `std::io::Error` and `serde_json::Error` (and play nicely with `anyhow`).
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// `true` if something exists at `path`.
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Ensure the parent directory of `path` exists, creating any missing
/// ancestors. No-op when there's no parent (e.g. a bare filename) or it
/// already exists.
pub fn ensure_parent_dir<P: AsRef<Path>>(path: P) -> Result<(), BoxError> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Read and deserialize JSON from `path` into `T`.
pub fn read_json<T, P>(path: P) -> Result<T, BoxError>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}

/// Serialize `value` to pretty-printed JSON at `path`, creating the parent
/// directory if needed. Overwrites any existing file.
pub fn write_json<T, P>(path: P, value: &T) -> Result<(), BoxError>
where
    T: Serialize,
    P: AsRef<Path>,
{
    ensure_parent_dir(&path)?;
    let writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(writer, value)?;
    Ok(())
}

/// Like [`write_json`] but emits compact, single-line JSON (smaller/faster,
/// good for machine-only files).
pub fn write_json_compact<T, P>(path: P, value: &T) -> Result<(), BoxError>
where
    T: Serialize,
    P: AsRef<Path>,
{
    ensure_parent_dir(&path)?;
    let writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer(writer, value)?;
    Ok(())
}

/// Read JSON from `path` if it exists; otherwise build a value with `default`,
/// write it to disk, and return it. Ideal for config/state files that should
/// be auto-created on first run.
pub fn read_or_create<T, P, F>(path: P, default: F) -> Result<T, BoxError>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<Path>,
    F: FnOnce() -> T,
{
    if path.as_ref().exists() {
        read_json(path)
    } else {
        let value = default();
        write_json(&path, &value)?;
        Ok(value)
    }
}

/// Read JSON from `path`, mutate it via `f`, write it back, and return the
/// updated value. Errors if the file doesn't exist.
pub fn update_json<T, P, F>(path: P, f: F) -> Result<T, BoxError>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<Path>,
    F: FnOnce(&mut T),
{
    let mut value: T = read_json(&path)?;
    f(&mut value);
    write_json(&path, &value)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Meta {
        name: String,
        retries: u32,
        paths: Vec<String>,
    }

    #[test]
    fn round_trip_and_auto_create() {
        let dir = std::env::temp_dir().join("json_util_test");
        let path = dir.join("config.json");
        let _ = fs::remove_file(&path); // start clean

        // File is missing -> default is written and returned.
        let cfg: Meta = read_or_create(&path, Meta::default).unwrap();
        assert_eq!(cfg, Meta::default());
        assert!(exists(&path));

        // Mutate on disk, then read it back.
        let updated = update_json(&path, |c: &mut Meta| c.retries = 5).unwrap();
        assert_eq!(updated.retries, 5);
        let reloaded: Meta = read_json(&path).unwrap();
        assert_eq!(reloaded.retries, 5);

        let _ = fs::remove_file(&path);
    }
}
