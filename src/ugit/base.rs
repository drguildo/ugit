use std::{fs, path::Component, path::Path};

/// Traverse a directory hierarchy, adding any files or directories to the object store.
pub fn write_tree(path: &Path) -> Option<String> {
    if is_ignored(path) {
        return None;
    }

    let mut entries: Vec<(&str, String, String)> = vec![];
    for dir_entry in fs::read_dir(&path).expect("Failed to read directory") {
        let path = dir_entry.expect("Failed to read directory entry").path();
        if path.is_file() {
            let contents = std::fs::read(&path).expect("Failed to read file contents");
            let oid = super::data::hash_object(&contents, "blob");
            let file_name = path
                .file_name()
                .expect("Failed to get file name for path")
                .to_str()
                .expect("Failed to convert OS string to string")
                .to_owned();
            entries.push(("blob", oid, file_name));
        } else if path.is_dir() {
            if let Some(oid) = write_tree(&path) {
                let dir_name = path
                    .file_name()
                    .expect("Failed to get directory name for path")
                    .to_str()
                    .expect("Failed to convert OS string to string")
                    .to_owned();
                entries.push(("tree", oid, dir_name));
            }
        }
    }

    let mut tree = String::new();
    for (object_type, oid, path) in entries {
        let tree_row = format!("{} {} {}\n", object_type, oid, path);
        tree.push_str(tree_row.as_str());
    }
    let oid = super::data::hash_object(&tree.into_bytes(), "tree");

    Some(oid)
}

/// Whether or not the specified path should not be added to the object store.
fn is_ignored(path: &Path) -> bool {
    path.components()
        .any(|c| c == Component::Normal(".ugit".as_ref()))
}
