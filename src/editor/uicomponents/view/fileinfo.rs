use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

use crate::editor::filetype::FileType;

#[derive(Default, Debug)]
pub struct FileInfo {
    path: Option<PathBuf>,
    filetype: FileType,
}

impl FileInfo {
    pub fn from(filename: &str) -> Self {
        let path = PathBuf::from(filename);
        let filetype = if path
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("rs"))
        {
            FileType::Rust
        } else {
            FileType::Text
        };

        Self {
            path: Some(path),
            filetype,
        }
    }

    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub const fn has_path(&self) -> bool {
        self.path.is_some()
    }

    pub const fn get_filetype(&self) -> FileType {
        self.filetype
    }
}

impl Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .get_path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");
        write!(formatter, "{name}")
    }
}
