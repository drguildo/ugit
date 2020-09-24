use std::{fs, path::Component, path::Path};

/// Traverse a directory hierarchy, adding any files or directories to the object store.
pub fn write_tree(path: &Path) {
    if is_ignored(path) {
        return;
    }

    for entry in fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_file() {
            let contents = std::fs::read(path).expect("Failed to read file contents");
            let oid = super::data::hash_object(&contents, "blob");
            println!("{}", oid);
        } else if path.is_dir() {
            write_tree(&path);
        }
    }
}

/// Whether or not the specified path should not be added to the object store.
fn is_ignored(path: &Path) -> bool {
    path.components()
        .any(|c| c == Component::Normal(".ugit".as_ref()))
}
