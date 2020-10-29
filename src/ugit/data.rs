use std::{fmt::Write as _, fs, io::Write as _, path::Path, path::PathBuf};

use sha1::{Digest, Sha1};

use super::UGIT_DIR;

/// Create a new ugit repository.
pub fn init() {
    let mut path = PathBuf::from(UGIT_DIR);
    fs::create_dir(&path).expect("Failed to create .ugit directory");

    path.push("objects");
    fs::create_dir(&path).expect("Unable to create objects directory");
}

/// Adds a new object to the object store and return it's OID.
pub fn hash_object(data: &[u8], object_type: &str) -> String {
    let oid = generate_oid(data);

    // Write the data to a file, using the OID as the filename.
    let path: PathBuf = get_object_path(&oid);
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
    let path: PathBuf = get_object_path(oid);
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

/// Map the specified reference to the specified OID.
pub fn update_ref(reference: &str, oid: &str) {
    let ref_path = std::path::Path::new(reference);
    assert!(!ref_path.is_absolute());

    let mut path = PathBuf::from(UGIT_DIR);
    path.push(reference);
    fs::create_dir_all(path.parent().unwrap())
        .expect("Failed to create reference directory structure");
    fs::write(path, oid).expect("Failed to update reference");
}

/// Retrieves the OID that the specified reference is mapped to.
pub fn get_ref(reference: &str) -> Option<String> {
    let mut path = PathBuf::from(UGIT_DIR);
    path.push(reference);
    if path.exists() {
        let data = fs::read(path).expect("Failed to read reference");
        Some(String::from_utf8(data).expect("Failed to convert reference data to OID"))
    } else {
        None
    }
}

pub fn get_refs() -> Vec<(String, String)> {
    let mut refs_path = PathBuf::from(UGIT_DIR);
    refs_path.push("refs");

    let mut ref_names = find_ref_names(&refs_path);
    ref_names.push("HEAD".to_string());

    let mut refs_to_oid: Vec<(String, String)> = vec![];
    for ref_name in ref_names {
        let oid = get_ref(&ref_name).expect("Failed to get OID for reference");
        refs_to_oid.push((ref_name, oid));
    }

    refs_to_oid
}

fn find_ref_names(path: &Path) -> Vec<String> {
    let mut ref_names: Vec<String> = vec![];

    let dir = std::fs::read_dir(path).unwrap();
    for dir_entry in dir {
        let dir_entry_path = dir_entry.unwrap().path();
        if dir_entry_path.is_file() {
            let ref_name = path_to_ref_name(&dir_entry_path);
            ref_names.push(ref_name);
        } else if dir_entry_path.is_dir() {
            let mut subdir_ref_names = find_ref_names(&dir_entry_path);
            ref_names.append(subdir_ref_names.as_mut());
        }
    }

    ref_names
}

fn path_to_ref_name(path: &Path) -> String {
    let ref_name = path
        .components()
        .skip(1) // Don't include the ugit directory in the generated name.
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect::<Vec<&str>>()
        .join("/");
    ref_name
}

/// Generates an OID from a byte vector.
fn generate_oid(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut oid = String::new();
    for byte in result {
        write!(&mut oid, "{:02x}", byte).expect("Unable to construct object filename");
    }
    oid
}

/// Return the path to an object in the object database.
fn get_object_path(oid: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(UGIT_DIR);
    path.push("objects");
    path.push(oid);
    path
}
