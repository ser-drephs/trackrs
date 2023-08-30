use std::{path::{PathBuf, Path}, str::FromStr};

#[derive(Clone, Debug, Default)]
pub struct Folder(PathBuf);

impl Folder {
    fn new(path: PathBuf) -> Folder {
        Folder(path)
    }

    pub fn join(&self, str: &str) -> PathBuf {
        self.0.join(str)
    }
}

impl From<PathBuf> for Folder {
    fn from(path: PathBuf) -> Self {
        Folder::new(path)
    }
}

impl From<&str> for Folder {
    fn from(str: &str) -> Self {
        Folder::new(PathBuf::from_str(str).unwrap())
    }
}

impl From<String> for Folder {
    fn from(str: String) -> Self {
        Folder::new(PathBuf::from_str(&str).unwrap())
    }
}

impl From<Folder> for PathBuf {
    fn from(val: Folder) -> Self {
        val.0.to_path_buf()
    }
}

impl AsRef<Path> for Folder {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}
