use std::{path::{PathBuf, Path}, str::FromStr};

#[derive(Debug, Default, Clone)]
pub struct Folder {
    inner: PathBuf,
}

impl Folder {
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.inner.join(path)
    }

    pub fn exists(&self) -> bool {
        self.inner.exists()
    }
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

impl From<&Folder> for PathBuf {
    fn from(val: &Folder) -> Self {
        val.inner.to_owned()
    }
}

impl AsRef<Path> for Folder {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
