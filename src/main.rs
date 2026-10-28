#![allow(clippy::all)]
use std::{
    collections::HashMap, env::args, fs::read, path::PathBuf, thread::sleep, time::Duration,
};

use sha2::{Digest, Sha256};

fn main() {
    let args: Vec<String> = args().collect();
    let path = PathBuf::from(&args[1]);

    let mut last_hashes = gather_hashes(path.clone());
    loop {
        let hash = gather_hashes(path.clone());
        if hash != last_hashes {
            println!("changed");
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
