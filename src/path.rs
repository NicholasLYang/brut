use camino::{Utf8Path, Utf8PathBuf};
use clean_path::clean;
use std::ops::{Deref, DerefMut};

pub struct RelativePath(Utf8Path);
pub struct RelativePathBuf(Utf8PathBuf);

pub struct AbsolutePath(Utf8Path);
pub struct AbsolutePathBuf(Utf8PathBuf);

impl Deref for RelativePath {
    type Target = Utf8Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for RelativePathBuf {
    type Target = Utf8Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativePathBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for AbsolutePath {
    type Target = Utf8Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AbsolutePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for AbsolutePathBuf {
    type Target = Utf8PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AbsolutePathBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AbsolutePathBuf {
    /// Checks if a path is absolute and if not, appends on `cwd`
    pub fn from_unknown(path: &Utf8Path, cwd: &Utf8Path) -> Self {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd.join(path)
        };

        AbsolutePathBuf(clean(path).try_into().unwrap())
    }
}

impl RelativePathBuf {
    /// Checks if path is relative and if not, strips `cwd`. Errors if `cwd` is not a prefix of `path`
    pub fn from_unknown(path: &Utf8Path, cwd: &Utf8Path) -> Result<Self, anyhow::Error> {
        let cleaned_path: Utf8PathBuf = clean(path).try_into()?;

        if cleaned_path.is_absolute() {
            Ok(RelativePathBuf(path.strip_prefix(cwd)?.to_path_buf()))
        } else {
            Ok(RelativePathBuf(cleaned_path))
        }
    }
}
