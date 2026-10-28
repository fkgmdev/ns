use std::{collections::HashMap, fs::read, path::PathBuf};

use sha2::{Digest, Sha256};

pub fn gather_hashes(path: PathBuf) -> HashMap<PathBuf, String> {
    let mut result = HashMap::new();
    let root_path = path.clone();
    path.read_dir().unwrap().for_each(|element| {
        if let Ok(subpath) = element {
            if subpath.path().is_dir() {
                let subresult = gather_hashes(subpath.path());
                result.extend(subresult);
            } else {
                let mut hasher = Sha256::new();
                hasher.update(read(subpath.path()).unwrap());
                let hash_result = hasher.finalize();
                let encoded = hex::encode(hash_result);
                result.insert(
                    subpath
                        .path()
                        .strip_prefix(&root_path)
                        .unwrap_or(&subpath.path())
                        .to_path_buf(),
                    encoded,
                );
            }
        }
    });
    result
}
pub struct Diffs {
    pub modified: Vec<PathBuf>,
    pub new: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}
pub fn calculate_changes(old: &HashMap<PathBuf, String>, new: &HashMap<PathBuf, String>) -> Diffs {
    let mut new_files = Vec::new();
    let mut changed_files = Vec::new();
    let mut deleted_files = Vec::new();

    for (path, hash) in new {
        match (old.get(path), hash) {
            (None, _) => new_files.push(path.clone()),
            (Some(old_hash), new_hash) => {
                if old_hash != new_hash {
                    changed_files.push(path.clone());
                }
            }
        }
    }
    for path in old.keys() {
        if new.get(path).is_none() {
            deleted_files.push(path.clone());
        }
    }
    Diffs {
        modified: changed_files,
        new: new_files,
        deleted: deleted_files,
    }
}
