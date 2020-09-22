use std::{fs, path::Component, path::Path};

pub fn write_tree(path: &Path) {
    if is_ignored(path) {
        return;
    }

    for entry in fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            write_tree(&path);
        } else {
            println!("{:?}", path);
        }
    }
}

fn is_ignored(path: &Path) -> bool {
    path.components()
        .any(|c| c == Component::Normal(".ugit".as_ref()))
}
