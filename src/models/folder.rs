use std::{ path::PathBuf, str::FromStr };

#[derive(Debug, Default, Clone)]
pub struct Folder {
    inner: PathBuf,
}

impl From<PathBuf> for Folder {
    fn from(path: PathBuf) -> Self {
        Folder { inner: path }
    }
}

impl From<&str> for Folder {
    fn from(str: &str) -> Self {
        Folder {
            inner: PathBuf::from_str(str).unwrap(),
        }
    }
}

impl From<String> for Folder {
    fn from(str: String) -> Self {
        Folder {
            inner: PathBuf::from_str(&str).unwrap(),
        }
    }
}

impl From<Folder> for PathBuf {
    fn from(val: Folder) -> Self {
        val.inner
    }
}
