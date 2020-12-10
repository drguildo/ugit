use std::{fmt::Write as _, fs, io::Write as _, path::Path, path::PathBuf};

use sha1::{Digest, Sha1};
use walkdir::WalkDir;

use super::UGIT_DIR;

#[derive(Debug)]
pub struct RefValue {
    pub symbolic: bool,
    pub value: Option<String>,
}

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

/// Map the specified reference to the specified value.
pub fn update_ref(reference: &str, value: &RefValue, deref: bool) {
    let reference = get_ref_internal(reference, deref).0;

    assert!(value.value.is_some());
    let new_ref_value = if value.symbolic {
        format!("ref: {}", value.value.as_ref().unwrap())
    } else {
        value.value.to_owned().unwrap()
    };

    let mut path = PathBuf::from(UGIT_DIR);
    path.push(reference);
    fs::create_dir_all(path.parent().unwrap())
        .expect("Failed to create reference directory structure");
    fs::write(path, new_ref_value).expect("Failed to update reference");
}

/// Retrieves the OID that the specified reference is mapped to.
pub fn get_ref(reference: &str, deref: bool) -> RefValue {
    let ref_value = get_ref_internal(reference, deref);
    ref_value.1
}

fn get_ref_internal(reference: &str, deref: bool) -> (String, RefValue) {
    let mut ref_path = PathBuf::from(UGIT_DIR);
    ref_path.push(reference);

    let mut value: Option<String> = None;

    let reference: String = reference.to_owned();
    if ref_path.is_file() {
        let ref_string = fs::read_to_string(ref_path).expect("Failed to read reference");
        value = Some(ref_string);
    }

    let symbolic = value.as_ref().map_or(false, |s| s.starts_with("ref:"));
    if symbolic {
        value = value.as_ref().map(|s| s.replacen("ref: ", "", 1));
        if deref {
            return get_ref_internal(&value.unwrap(), true);
        }
    }
    return (reference, RefValue { symbolic, value });
}

pub fn delete_ref(reference: &str, deref: bool) {
    let reference = get_ref_internal(reference, deref).0;
    let mut ref_path = PathBuf::from(UGIT_DIR);
    ref_path.push(reference);
    fs::remove_file(ref_path).expect("Failed to delete ref");
}

pub fn get_refs(prefix: Option<&str>, deref: bool) -> Vec<(String, RefValue)> {
    let mut refs_path = PathBuf::from(UGIT_DIR);
    refs_path.push("refs");

    let mut ref_names = find_ref_names(&refs_path);
    ref_names.push("HEAD".to_string());
    ref_names.push("MERGE_HEAD".to_string());

    let mut refs_to_values: Vec<(String, RefValue)> = vec![];
    for ref_name in ref_names {
        if let Some(prefix) = prefix {
            if !ref_name.starts_with(prefix) {
                continue;
            }
        }
        let ref_value = get_ref(&ref_name, deref);
        if ref_value.value.is_some() {
            // This is mainly to handle MERGE_HEAD which will only exist if we're in the middle of a
            // merge.
            refs_to_values.push((ref_name, ref_value));
        }
    }

    refs_to_values
}

fn find_ref_names(path: &Path) -> Vec<String> {
    let mut ref_names: Vec<String> = vec![];

    for entry in WalkDir::new(path) {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                let ref_name = path_to_ref_name(entry.path());
                ref_names.push(ref_name);
            }
        }
    }

    ref_names
}

fn path_to_ref_name(path: &Path) -> String {
    path.components()
        .skip(1) // Don't include the ugit directory in the generated name.
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect::<Vec<&str>>()
        .join("/")
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
