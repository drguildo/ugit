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
                .or_insert(vec![None; trees.len()]);
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
