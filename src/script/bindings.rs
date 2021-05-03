/*! Utils for implementing scripting language bindings */

use std::{fs, io, path::PathBuf};

pub struct ScriptFile {
    pub path: String,
    pub code: String,
}

// Allow dead code in case all languages are disabled:
#[allow(dead_code)]
impl ScriptFile {
    pub fn read_from(path: PathBuf) -> io::Result<Self> {
        let code = fs::read_to_string(&path)?;
        let path_string = path.to_string_lossy().to_string();
        return Ok(Self {
            code,
            path: path_string,
        });
    }
}
