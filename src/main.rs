#![allow(clippy::all)]
use ns::diffengine::{calculate_changes, gather_hashes};
use std::{env::args, path::PathBuf, thread::sleep, time::Duration};

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
