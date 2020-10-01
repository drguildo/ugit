use std::{
    ffi, fs,
    path::Component,
    path::{self, Path},
};

/// Traverses a directory hierarchy, adding any files or directories to the object store.
pub fn write_tree(path: &Path) -> Option<String> {
    if is_ignored(path) {
        return None;
    }

    let mut entries: Vec<(&str, String, ffi::OsString)> = vec![];
    for dir_entry in fs::read_dir(&path).expect("Failed to read directory") {
        let path = dir_entry.expect("Failed to read directory entry").path();
        if path.is_file() {
            let contents = std::fs::read(&path).expect("Failed to read file contents");
            let oid = super::data::hash_object(&contents, "blob");
            let file_name = path
                .file_name()
                .expect("Failed to get file name for path")
                .to_owned();
            entries.push(("blob", oid, file_name));
        } else if path.is_dir() {
            if let Some(oid) = write_tree(&path) {
                let dir_name = path
                    .file_name()
                    .expect("Failed to get directory name for path")
                    .to_owned();
                entries.push(("tree", oid, dir_name));
            }
        }
    }

    let mut tree = String::new();
    for (object_type, oid, path) in entries {
        let path_string = path.to_str().expect("Failed to convert path to string");
        let tree_row = format!("{} {} {}\n", object_type, oid, path_string);
        tree.push_str(tree_row.as_str());
    }
    let oid = super::data::hash_object(&tree.into_bytes(), "tree");

    Some(oid)
}

/// Retrieves the tree with the specified OID from the object store and writes it to the current
/// directory.
pub fn read_tree(tree_oid: &str) {
    let tree = get_tree(tree_oid, None);
    for (oid, path) in tree {
        let directories = Path::new(&path)
            .parent()
            .expect("Failed to get parent directories from path");
        // Check whether the file is contained in a subdirectory.
        // XXX(sjm): Is there a nicer way of doing this?
        if directories != Path::new("") {
            std::fs::create_dir_all(directories).expect("Failed to create parent directories");
        }

        let contents = super::data::get_object(oid.as_str(), None);
        std::fs::write(path, contents).expect("Failed to write file contents");
    }
}

/// Recursively traverses the tree with the specified OID and returns a flattened list of file OIDs
/// and their paths.
fn get_tree(oid: &str, base_path: Option<&str>) -> Vec<(String, ffi::OsString)> {
    let tree_object = super::data::get_object(oid, Some("tree"));
    let tree = std::str::from_utf8(&tree_object).expect("Tree is not valid UTF-8");

    let base_path = base_path.unwrap_or("");

    let mut result: Vec<(String, ffi::OsString)> = vec![];
    for line in tree.lines() {
        let split: Vec<&str> = line.split_whitespace().collect();

        let object_type = split
            .get(0)
            .expect("Failed to get object type from tree object");
        let oid = split.get(1).expect("Failed to get OID from tree object");
        let relative_path = split.get(2).expect("Failed to get path from tree object");

        let mut path = path::PathBuf::new();
        path.push(base_path);
        path.push(relative_path);

        assert!(!is_illegal(&path));

        match *object_type {
            "blob" => {
                result.push((oid.to_string(), path.into_os_string()));
            }
            "tree" => {
                let subtree = get_tree(oid, path.to_str());
                for subtree_object in subtree {
                    result.push(subtree_object);
                }
            }
            _ => panic!(format!("Unrecognised object type: {}", *object_type)),
        }
    }
    result
}

/// Whether or not the specified path should not be added to the object store.
fn is_ignored(path: &Path) -> bool {
    path.components()
        .any(|c| c == Component::Normal(".ugit".as_ref()))
}

// Whether a path contains illegal components.
fn is_illegal(path: &Path) -> bool {
    let illegal_path_components = [Component::RootDir, Component::CurDir, Component::ParentDir];
    path.components()
        .any(|c| illegal_path_components.contains(&c))
}
