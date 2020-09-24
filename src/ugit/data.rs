use std::{fmt::Write as _, fs};
use std::{io::Write as _, path::PathBuf};

use sha1::{digest, Digest, Sha1};

const GIT_DIR: &str = ".ugit";

/// Create a new ugit repository.
pub fn init() {
    let mut path = PathBuf::from(GIT_DIR);
    fs::create_dir(&path).expect("Failed to create .ugit directory");

    path.push("objects");
    fs::create_dir(&path).expect("Unable to create objects directory");
}

/// Add a new object to the object store and return it's OID.
pub fn hash_object(data: &Vec<u8>, object_type: &str) -> String {
    // TODO(sjm): Add a hash_object overload that only takes the data and just calls this function
    // with type "blob"?

    // Hex encode the SHA1 of the data to get the OID.
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result: digest::generic_array::GenericArray<u8, _> = hasher.finalize();
    let mut oid = String::new();
    for byte in result {
        write!(&mut oid, "{:x}", byte).expect("Unable to construct object filename");
    }

    // Write the data to a file, using the OID as the filename.
    let path: PathBuf = [GIT_DIR, "objects", &oid].iter().collect();
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

/// Retrieve the object with the specified OID from the object store.
pub fn get_object(oid: &str, expected_type: Option<&str>) -> Vec<u8> {
    let path: PathBuf = [GIT_DIR, "objects", oid].iter().collect();
    let contents = fs::read(path).expect("Failed to read file data");

    // Find the index of the null byte that separates the object type from the data.
    let index = contents
        .iter()
        .position(|x| *x == 0)
        .expect("Failed to find null byte object type separator");
    // Split the data on the null byte.
    let (object_type, data) = contents.split_at(index);

    if let Some(expected_type) = expected_type {
        // Check whether the object type stored in the data is the expected type.
        assert!(expected_type.as_bytes() == object_type);
    }

    // NOTE(sjm): Does this copy to a new Vec? If so, is it possible to do this without copying?
    data.to_vec()
}
