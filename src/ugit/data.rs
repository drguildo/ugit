use std::{fmt::Write as _, fs};
use std::{io::Write as _, path::PathBuf};

use sha1::{Digest, Sha1};

/// Create a new ugit repository.
pub fn init() {
    let mut path = PathBuf::from(super::UGIT_DIR);
    fs::create_dir(&path).expect("Failed to create .ugit directory");

    path.push("objects");
    fs::create_dir(&path).expect("Unable to create objects directory");
}

/// Adds a new object to the object store and return it's OID.
pub fn hash_object(data: &Vec<u8>, object_type: &str) -> String {
    let oid = generate_oid(data);

    // Write the data to a file, using the OID as the filename.
    let path: PathBuf = [super::UGIT_DIR, "objects", &oid].iter().collect();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .expect("Failed to open object file for writing");
    file.write_all(object_type.as_bytes())
        .expect("Failed to write object type");
    file.write_all(b"\x00").expect("Failed to write null byte");
    file.write_all(data).expect("Failed to write file data");

    oid
}

/// Retrieves the object with the specified OID from the object store.
pub fn get_object(oid: &str, expected_type: Option<&str>) -> Vec<u8> {
    let path: PathBuf = [super::UGIT_DIR, "objects", oid].iter().collect();
    let contents = fs::read(path).expect("Failed to read file data");

    // Find the index of the null byte that separates the object type from the data.
    let index = contents
        .iter()
        .position(|b| *b == 0)
        .expect("Failed to find null byte object type separator");
    // Split the data on the null byte.
    let object_type = &contents[0..index];
    let data = &contents[index + 1..];

    if let Some(expected_type) = expected_type {
        // Check whether the object type stored in the data is the expected type.
        assert!(expected_type.as_bytes() == object_type);
    }

    data.to_vec()
}

/// Generates an OID from a byte vector.
fn generate_oid(bytes: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut oid = String::new();
    for byte in result {
        write!(&mut oid, "{:x}", byte).expect("Unable to construct object filename");
    }
    oid
}
