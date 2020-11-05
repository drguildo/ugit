use std::{
    collections::{HashSet, VecDeque},
    env, ffi, fs,
    path::Component,
    path::{self, Path},
};

use super::{data, Commit, UGIT_DIR};

pub fn get_oid(mut name: &str) -> String {
    if name == "@" {
        name = "HEAD";
    }

    let refs_to_try: Vec<String> = vec![
        format!("{}", name),
        format!("refs/{}", name),
        format!("refs/tags/{}", name),
        format!("refs/heads/{}", name),
    ];

    for reference in refs_to_try {
        let reference = data::get_ref(&reference);
        if reference.is_some() {
            // Name is a ref.
            return reference.unwrap();
        }
    }

    let is_hex = name.chars().all(|c| c.is_ascii_hexdigit());
    if name.len() == 40 && is_hex {
        // Name is an OID.
        return name.to_owned();
    }

    panic!(format!("Unknown name {}", name));
}

pub fn create_tag(name: &str, oid: &str) {
    let ref_path = format!("refs/tags/{}", name);
    data::update_ref(&ref_path, oid);
}

pub fn create_branch(name: &str, oid: &str) {
    let ref_path = format!("refs/heads/{}", name);
    data::update_ref(&ref_path, oid);
}

/// Store the contents of the current directory to the object database, creates a commit object and
/// updates the HEAD.
pub fn commit(message: &str) -> Option<String> {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let tree_oid = write_tree(&current_dir).expect("Failed to write tree");

    let mut commit = String::new();
    commit.push_str(format!("tree {}\n", tree_oid).as_str());
    if let Some(head_oid) = data::get_ref("HEAD") {
        commit.push_str(format!("parent {}\n", head_oid).as_str());
    }
    commit.push_str("\n");
    commit.push_str(message);

    let commit_oid = data::hash_object(&commit.as_bytes().to_vec(), "commit");
    data::update_ref("HEAD", &commit_oid);
    Some(commit_oid)
}

pub fn get_commit(oid: &str) -> Commit {
    let commit_data = data::get_object(oid, Some("commit"));
    let commit = String::from_utf8(commit_data).expect("Commit contains invalid data");
    let mut commit_lines = commit.lines();

    let mut tree_oid: Option<&str> = None;
    let mut parent_oid: Option<&str> = None;

    for line in commit_lines.by_ref().take_while(|l| *l != "") {
        let mut split_line = line.split_whitespace();
        let key = split_line
            .next()
            .expect("Failed to retrieve key from commit header");
        let oid = split_line.next();
        match key {
            "tree" => tree_oid = oid,
            "parent" => parent_oid = oid,
            _ => panic!("Unrecognised commit header type"),
        }
    }

    let message: String = commit_lines.collect();

    if let Some(tree_oid) = tree_oid {
        Commit {
            tree: tree_oid.to_string(),
            parent: parent_oid.map(ToOwned::to_owned),
            message,
        }
    } else {
        panic!("Commit did not contain tree header")
    }
}

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
            let oid = data::hash_object(&contents, "blob");
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
    let oid = data::hash_object(&tree.into_bytes(), "tree");

    Some(oid)
}

/// Retrieves the tree with the specified OID from the object store and writes it to the current
/// directory.
pub fn read_tree(tree_oid: &str) {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    empty_directory(&current_dir);

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

        let contents = data::get_object(oid.as_str(), None);
        std::fs::write(path, contents).expect("Failed to write file contents");
    }
}

/// Recursively traverses the tree with the specified OID and returns a flattened list of file OIDs
/// and their paths.
fn get_tree(oid: &str, base_path: Option<&str>) -> Vec<(String, ffi::OsString)> {
    let tree_object = data::get_object(oid, Some("tree"));
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

pub fn checkout(oid: &str) {
    let commit = get_commit(oid);
    read_tree(&commit.tree);
    data::update_ref("HEAD", oid);
}

/// Retrieve the OIDs of all the commits that are reachable from the commits with the specified
/// OIDs.
pub fn get_commits_and_parents(root_oids: Vec<&str>) -> Vec<String> {
    let mut oids_to_visit: VecDeque<String> = VecDeque::new();
    let mut visite_oids: HashSet<String> = HashSet::new();

    for oid in root_oids {
        oids_to_visit.push_back(oid.to_owned());
    }

    let mut oids: Vec<String> = vec![];
    while let Some(oid) = oids_to_visit.pop_back() {
        if visite_oids.contains(&oid) {
            continue;
        }

        let commit: Commit = get_commit(&oid);

        visite_oids.insert(oid.clone());

        oids.push(oid.clone());

        if let Some(parent_oid) = commit.parent {
            oids_to_visit.push_back(parent_oid);
        }
    }

    oids
}

/// Whether the specified path is a ugit repository. This is overly simplistic and should really
/// check whether the .ugit directory at least contains an objects sub-directory.
pub fn is_ugit_repository(path: &Path) -> bool {
    let mut ugit_data_dir = path.to_owned();
    ugit_data_dir.push(UGIT_DIR);
    ugit_data_dir.is_dir()
}

/// Whether or not the specified path should not be added to the object store.
fn is_ignored(path: &Path) -> bool {
    path.components()
        .any(|c| c == Component::Normal(UGIT_DIR.as_ref()))
}

/// Whether a path contains illegal components.
fn is_illegal(path: &Path) -> bool {
    let illegal_path_components = [Component::RootDir, Component::CurDir, Component::ParentDir];
    path.components()
        .any(|c| illegal_path_components.contains(&c))
}

/// Empty the specified directory of its contents, ignoring the ugit directory.
fn empty_directory(dir_path: &Path) {
    let dir = fs::read_dir(dir_path).expect("Failed to read directory");
    for entry in dir {
        let path = entry.expect("Failed to read directory entry").path();
        if path.is_dir() {
            if !is_ignored(&path) {
                empty_directory(&path);
                fs::remove_dir(path).expect("Failed to remove directory");
            }
        } else if path.is_file() {
            fs::remove_file(path).expect("Failed to remove file");
        }
    }
}
