use std::path::PathBuf;
use std::{fmt::Write, fs};

use sha1::{digest, Digest, Sha1};

const GIT_DIR: &str = ".ugit";

pub fn init() {
    let mut path = PathBuf::from(GIT_DIR);
    fs::create_dir(&path).expect("Failed to create .ugit directory");

    path.push("objects");
    fs::create_dir(&path).expect("Unable to create objects directory");
}

pub fn hash_object(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result: digest::generic_array::GenericArray<u8, _> = hasher.finalize();

    let mut oid = String::new();
    for byte in result {
        write!(&mut oid, "{:x}", byte).expect("Unable to construct object filename");
    }

    let path: PathBuf = [GIT_DIR, "objects", &oid].iter().collect();
    fs::write(path, data).expect("Failed to write file data");

    oid
}
