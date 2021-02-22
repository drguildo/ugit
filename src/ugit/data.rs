use std::{fmt::Write as _, fs, io::Write as _, path::Path, path::PathBuf};

use sha1::{Digest, Sha1};
use walkdir::WalkDir;

use super::DEFAULT_REPO;

#[derive(Debug)]
pub struct RefValue {
    pub symbolic: bool,
    pub value: Option<String>,
}

/// Create a new ugit repository.
pub fn init() {
    let mut path = PathBuf::from(DEFAULT_REPO);
    fs::create_dir(&path).expect("Failed to create .ugit directory");

    path.push("objects");
    fs::create_dir(&path).expect("Unable to create objects directory");
}

/// Adds a new object to the object store and return it's OID.
pub fn hash_object(data: &[u8], object_type: &str) -> String {
    let oid = generate_oid(data);

    // Write the data to a file, using the OID as the filename.
    let path: PathBuf = get_object_path(&PathBuf::from(DEFAULT_REPO), &oid);
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
pub fn get_object(repo_path: &Path, oid: &str, expected_type: Option<&str>) -> Vec<u8> {
    let path: PathBuf = get_object_path(repo_path, oid);
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
pub fn update_ref(repo_path: &Path, reference: &str, value: &RefValue, deref: bool) {
    let reference = get_ref_internal(repo_path, reference, deref).0;

    assert!(value.value.is_some());
    let new_ref_value = if value.symbolic {
        format!("ref: {}", value.value.as_ref().unwrap())
    } else {
        value.value.to_owned().unwrap()
    };

    let mut ref_path = PathBuf::from(repo_path);
    ref_path.push(reference);
    fs::create_dir_all(ref_path.parent().unwrap())
        .expect("Failed to create reference directory structure");
    fs::write(ref_path, new_ref_value).expect("Failed to update reference");
}

/// Retrieves the OID that the specified reference is mapped to.
pub fn get_ref(repo_path: &Path, reference: &str, deref: bool) -> RefValue {
    let ref_value = get_ref_internal(repo_path, reference, deref);
    ref_value.1
}

fn get_ref_internal(repo_path: &Path, reference: &str, deref: bool) -> (String, RefValue) {
    let mut ref_path = PathBuf::from(repo_path);
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
            return get_ref_internal(repo_path, &value.unwrap(), true);
        }
    }
    return (reference, RefValue { symbolic, value });
}

pub fn delete_ref(reference: &str, deref: bool) {
    let reference = get_ref_internal(&PathBuf::from(DEFAULT_REPO), reference, deref).0;
    let mut ref_path = PathBuf::from(DEFAULT_REPO);
    ref_path.push(reference);
    fs::remove_file(ref_path).expect("Failed to delete ref");
}

pub fn get_refs(repo_path: &Path, prefix: Option<&str>, deref: bool) -> Vec<(String, RefValue)> {
    let mut refs_path = PathBuf::from(repo_path);
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
        let ref_value = get_ref(repo_path, &ref_name, deref);
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
                let ref_name = entry.path().strip_prefix(path.parent().unwrap()).unwrap();
                ref_names.push(ref_name.as_os_str().to_str().unwrap().to_owned());
            }
        }
    }

    ref_names
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
fn get_object_path(repo_path: &Path, oid: &str) -> PathBuf {
    let mut path = PathBuf::from(repo_path);
    path.push("objects");
    path.push(oid);
    path
}

fn object_exists(oid: &str) -> bool {
    let mut object_path = PathBuf::from(DEFAULT_REPO);
    object_path.push("objects");
    object_path.push(oid);
    object_path.exists()
}

pub fn fetch_object_if_missing(remote_path: &Path, oid: &str) {
    if object_exists(oid) {
        return;
    }

    let mut from = remote_path.to_path_buf();
    from.push("objects");
    from.push(oid);

    let mut to = PathBuf::new();
    to.push(DEFAULT_REPO);
    to.push("objects");
    to.push(oid);

    fs::copy(from, to).expect(&format!("Failed to copy remote object with OID {}", oid));
}

pub fn push_object(remote_path: &Path, oid: &str) {
    let mut local_object_path = PathBuf::from(DEFAULT_REPO);
    local_object_path.push("objects");
    local_object_path.push(oid);

    let mut remote_object_path = PathBuf::from(remote_path);
    remote_object_path.push("objects");
    remote_object_path.push(oid);

    fs::copy(local_object_path, remote_object_path)
        .expect(&format!("Failed to copy local object with OID {}", oid));
}
