use std::{collections::HashMap, ffi::OsString};

use super::Tree;

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

pub fn diff_trees(t_from: &Tree, t_to: &Tree) -> String {
    let mut output = String::new();
    for (path, oids) in compare_trees(&vec![t_from, t_to]) {
        let o_from = &oids[0];
        let o_to = &oids[1];
        if o_from != o_to {
            let path_string = path.to_str().expect("Failed to convert path to string");
            output.push_str(&format!("changed: {}\n", path_string));
        }
    }
    output
}
