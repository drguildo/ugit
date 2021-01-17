use std::io::Write;
use std::process::Command;
use std::{collections::HashMap, ffi::OsString};

use tempfile::NamedTempFile;

use super::{data, Tree};

fn diff_blobs(o_from: Option<&str>, o_to: Option<&str>, path: &str) -> String {
    let mut f_from = NamedTempFile::new().expect("Failed to create temp file");
    let mut f_to = NamedTempFile::new().expect("Failed to create temp file");

    if let Some(o_from) = o_from {
        let data = data::get_object(o_from, Some("blob"));
        let s = std::str::from_utf8(&data).expect("Failed to convert data to string");
        write!(f_from, "{}", s).expect("Failed to write to temp file");
    }

    if let Some(o_to) = o_to {
        let data = data::get_object(o_to, Some("blob"));
        let s = std::str::from_utf8(&data).expect("Failed to convert data to string");
        write!(f_to, "{}", s).expect("Failed to write to temp file");
    }

    let mut diff_command = Command::new("diff");
    diff_command.arg("--unified").arg("--show-c-function");
    diff_command
        .arg("--label")
        .arg(format!("a/{}", path))
        .arg(f_from.path().to_str().unwrap());
    diff_command
        .arg("--label")
        .arg(format!("b/{}", path))
        .arg(f_to.path().to_str().unwrap());

    let diff_output = diff_command.output().unwrap();

    std::str::from_utf8(&diff_output.stdout)
        .expect("Failed to convert diff output to string")
        .to_owned()
}

fn compare_trees(trees: &[&Tree]) -> HashMap<OsString, Vec<Option<String>>> {
    let mut entries: HashMap<OsString, Vec<Option<String>>> = HashMap::new();

    for (i, tree) in trees.iter().enumerate() {
        for (oid, path) in *tree {
            let oids = entries
                .entry(path.to_owned())
                .or_insert_with(|| vec![None; trees.len()]);
            oids[i] = Some(oid.clone());
        }
    }

    entries
}

pub fn get_changed_files(t_from: &Tree, t_to: &Tree) -> Vec<(OsString, &'static str)> {
    let mut result = vec![];

    for (path, oids) in compare_trees(&vec![t_from, t_to]) {
        let o_from = &oids[0];
        let o_to = &oids[1];
        if o_from != o_to {
            let action = if o_from.is_none() {
                "new file"
            } else if o_to.is_none() {
                "deleted"
            } else {
                "modified"
            };
            result.push((path, action));
        }
    }

    result
}

pub fn diff_trees(t_from: &Tree, t_to: &Tree) -> String {
    let mut output = String::new();
    for (path, oids) in compare_trees(&vec![t_from, t_to]) {
        let o_from = &oids[0];
        let o_to = &oids[1];
        if o_from != o_to {
            let path_string = path.to_str().expect("Failed to convert path to string");
            let diff = diff_blobs(o_from.as_deref(), o_to.as_deref(), path_string);
            output.push_str(&diff);
        }
    }
    output
}

pub fn merge_trees(t_base: &Tree, t_head: &Tree, t_other: &Tree) -> HashMap<OsString, String> {
    let mut tree = HashMap::new();
    for (path, oids) in compare_trees(&vec![t_base, t_head, t_other]) {
        let o_base = &oids[0];
        let o_head = &oids[1];
        let o_other = &oids[2];
        tree.insert(
            path,
            merge_blobs(o_base.as_deref(), o_head.as_deref(), o_other.as_deref()),
        );
    }
    tree
}

fn merge_blobs(o_base: Option<&str>, o_head: Option<&str>, o_other: Option<&str>) -> String {
    let f_base = NamedTempFile::new().expect("Failed to create temp file");
    let f_head = NamedTempFile::new().expect("Failed to create temp file");
    let f_other = NamedTempFile::new().expect("Failed to create temp file");

    if let Some(oid) = o_base {
        std::fs::write(&f_base, data::get_object(oid, Some("blob"))).expect("Failed to write blob");
    }

    if let Some(oid) = o_head {
        std::fs::write(&f_head, data::get_object(oid, Some("blob"))).expect("Failed to write blob");
    }

    if let Some(oid) = o_other {
        std::fs::write(&f_other, data::get_object(oid, Some("blob")))
            .expect("Failed to write blob");
    }

    let mut diff_command = Command::new("diff3");
    diff_command
        .arg("-m")
        .arg("-L")
        .arg("HEAD")
        .arg(f_head.path().to_str().unwrap())
        .arg("-L")
        .arg("BASE")
        .arg(f_base.path().to_str().unwrap())
        .arg("-L")
        .arg("MERGE_HEAD")
        .arg(f_other.path().to_str().unwrap());

    let diff_output = diff_command.output().unwrap();
    if let Some(return_code) = diff_output.status.code() {
        // 0: success
        // 1: conflicts
        assert!(return_code == 0 || return_code == 1);
    }
    let diff_string =
        std::str::from_utf8(&diff_output.stdout).expect("Failed to convert diff output to string");

    diff_string.to_owned()
}
