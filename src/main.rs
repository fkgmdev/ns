#![allow(clippy::all)]
use std::{
    collections::HashMap,
    env::args,
    fs::read,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use sha2::{Digest, Sha256};

fn main() {
    let args: Vec<String> = args().collect();
    let path = PathBuf::from(&args[1]);

    let mut last_hashes = gather_hashes(path.clone());
    loop {
        let hash = gather_hashes(path.clone());
        if hash != last_hashes {
            let diffs = calculate_changes(&last_hashes, &hash);
            for x in diffs.modified {
                println!("Modified: {}", x.display());
            }
            for x in diffs.new {
                println!("New: {}", x.display());
            }
            for x in diffs.deleted {
                println!("Deleted: {}", x.display());
            }
        }
        sleep(Duration::from_secs(1));
        last_hashes = hash;
    }
}

fn gather_hashes(path: PathBuf) -> HashMap<PathBuf, String> {
    let mut result = HashMap::new();
    for element in path.read_dir().unwrap() {
        if let Ok(subpath) = element {
            if subpath.path().is_dir() {
                let subresult = gather_hashes(subpath.path());
                result.extend(subresult);
            } else {
                let mut hasher = Sha256::new();
                hasher.update(read(subpath.path()).unwrap());
                let hash_result = hasher.finalize();
                let encoded = hex::encode(hash_result);
                result.insert(subpath.path(), encoded);
            }
        }
    }
    result
}
struct Diffs {
    modified: Vec<PathBuf>,
    new: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
}
fn calculate_changes(old: &HashMap<PathBuf, String>, new: &HashMap<PathBuf, String>) -> Diffs {
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
    for (path, _) in old {
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
