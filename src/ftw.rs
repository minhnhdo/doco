use std::fmt;
use std::path::Path;
use std::fs;
use std::result;

pub type Result = result::Result<bool, FileTraverseError>;
pub type FileCallback = fn(&Path) -> Result;

struct NotADirectory {
    name: String,
}

pub struct FileTraverseError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for NotADirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Not a directory: {}", self.name)
    }
}

impl fmt::Display for FileTraverseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error processing file {} ({})", self.path, self.message)
    }
}

fn ftw_rec(path: &Path, on_dir: fn(&Path) -> Result, on_file: FileCallback) -> Result {
    let path_str = String::from(path.to_str().unwrap());

    if path.is_dir() {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(err) => {
                return Err(FileTraverseError {
                               path: path_str.clone(),
                               message: format!("{}", err),
                           })
            }
        };

        for entry in entries {
            let entry = entry.unwrap();
            let curr_path = entry.path();

            if curr_path.is_dir() {
                on_dir(&curr_path)?;
                ftw_rec(&curr_path, on_dir, on_file)?;
            } else {
                on_file(&curr_path)?;
            }
        }
    }

    Ok(true)
}

pub fn ftw(path: &String, on_dir: FileCallback, on_file: FileCallback) -> Result {
    let p = Path::new(path);
    if !p.is_dir() {
        return Err(FileTraverseError {
                       path: path.clone(),
                       message: format!("{}", NotADirectory { name: path.clone() }),
                   });
    }

    ftw_rec(p, on_dir, on_file)?;
    Ok(true)
}
