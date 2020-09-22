use std::{fs, path::Path};

pub fn write_tree(path: &Path) {
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
