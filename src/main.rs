use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process,
};

use clap::{Arg, Command};

mod ugit;
use ugit::{base, data, diff, DEFAULT_REPO};

fn main() {
    const ABOUT_INIT: &str = "Create a new ugit repository";
    const ABOUT_HASH_OBJECT: &str = "Add the specified file to the object store and print its OID";
    const ABOUT_CAT_FILE: &str = "Print the contents of the file with the specified OID";
    const ABOUT_WRITE_TREE: &str = "Write the current directory to the object store";
    const ABOUT_READ_TREE: &str =
        "Replace the contents of the current directory with the tree with the specified OID";
    const ABOUT_COMMIT: &str = "Commit the current directory";
    const ABOUT_LOG: &str = "Print the commit history, optionally beginning at the specified OID";
    const ABOUT_CHECKOUT: &str =
        "Restore the working tree to that of the commit with the specified OID";
    const ABOUT_TAG: &str = "Create a reference with the specified name";
    const ABOUT_BRANCH: &str = "List the available branches, or create a new one";
    const ABOUT_STATUS: &str = "Print the currently checked out branch";

    let matches = Command::new(clap::crate_name!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .subcommand(Command::new("init").about(ABOUT_INIT))
        .subcommand(
            Command::new("hash-object")
                .about(ABOUT_HASH_OBJECT)
                .arg(Arg::new("filename").required(true)),
        )
        .subcommand(
            Command::new("cat-file")
                .about(ABOUT_CAT_FILE)
                .arg(Arg::new("oid").required(true)),
        )
        .subcommand(Command::new("write-tree").about(ABOUT_WRITE_TREE))
        .subcommand(
            Command::new("read-tree")
                .about(ABOUT_READ_TREE)
                .arg(Arg::new("tree_oid").required(true)),
        )
        .subcommand(
            Command::new("commit").about(ABOUT_COMMIT).arg(
                Arg::new("message")
                    .short('m')
                    .long("message")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("log")
                .about(ABOUT_LOG)
                .arg(Arg::new("commit_oid").default_value("@")),
        )
        .subcommand(Command::new("show").arg(Arg::new("commit_oid").default_value("@")))
        .subcommand(
            Command::new("checkout")
                .about(ABOUT_CHECKOUT)
                .arg(Arg::new("commit").required(true)),
        )
        .subcommand(
            Command::new("tag")
                .about(ABOUT_TAG)
                .arg(Arg::new("name").required(true))
                .arg(Arg::new("oid").default_value("@")),
        )
        .subcommand(Command::new("k"))
        .subcommand(
            Command::new("branch")
                .about(ABOUT_BRANCH)
                .arg(Arg::new("name"))
                .arg(Arg::new("start_point").default_value("@")),
        )
        .subcommand(Command::new("status").about(ABOUT_STATUS))
        .subcommand(Command::new("reset").arg(Arg::new("oid").required(true)))
        .subcommand(Command::new("diff").arg(Arg::new("commit").default_value("@")))
        .subcommand(Command::new("merge").arg(Arg::new("commit").default_value("@")))
        .subcommand(
            Command::new("merge-base")
                .arg(Arg::new("commit1").required(true))
                .arg(Arg::new("commit2").required(true)),
        )
        .subcommand(Command::new("fetch").arg(Arg::new("remote").required(true)))
        .subcommand(
            Command::new("push")
                .arg(Arg::new("remote").required(true))
                .arg(Arg::new("branch").required(true)),
        )
        .arg_required_else_help(true)
        .get_matches();

    if matches.subcommand_matches("init").is_some() {
        base::init();
        process::exit(0);
    }

    // All of the subsequent subcommands need to be run within an existing ugit repository, so exit
    // with an error if the current working directory isn't one.
    exit_if_not_repository();

    let default_repo = &PathBuf::from(DEFAULT_REPO);

    if let Some(matches) = matches.subcommand_matches("hash-object") {
        let filename = matches.get_one::<String>("filename").unwrap();
        let contents = fs::read(filename).expect("Failed to read file contents");
        let object_hash = data::hash_object(&contents, "blob");
        io::stdout()
            .write_all(object_hash.as_bytes())
            .expect("Failed to output object ID");
        io::stdout()
            .write_all(b"\n")
            .expect("Failed to output newline");
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("cat-file") {
        if let Some(oid) = base::get_oid(matches.get_one::<String>("oid").unwrap()) {
            let contents = data::get_object(default_repo, &oid, None);
            io::stdout()
                .write_all(&contents)
                .expect("Failed to output file data");
        }
        process::exit(0);
    }

    if matches.subcommand_matches("write-tree").is_some() {
        let cwd = env::current_dir().expect("Failed to get current working directory");
        base::write_tree(cwd.as_path());
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("read-tree") {
        if let Some(tree_oid) = base::get_oid(matches.get_one::<String>("tree_oid").unwrap()) {
            base::read_tree(default_repo, &tree_oid);
        }
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("commit") {
        let message = matches.get_one::<String>("message").unwrap();
        base::commit(message);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("log") {
        if let Some(commit_oid) = base::get_oid(matches.get_one::<String>("commit_oid").unwrap()) {
            log(&commit_oid);
        }
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("show") {
        let oid = base::get_oid(matches.get_one::<String>("commit_oid").unwrap());
        show(oid.as_deref());
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("checkout") {
        let commit = matches.get_one::<String>("commit").unwrap();
        base::checkout(&commit);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("tag") {
        let name = matches.get_one::<String>("name").unwrap();
        if let Some(oid) = base::get_oid(matches.get_one::<String>("oid").unwrap()) {
            base::create_tag(name, &oid);
        }
        process::exit(0);
    }

    if matches.subcommand_matches("k").is_some() {
        k();
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("branch") {
        if let Some(name) = matches.get_one::<String>("name") {
            if let Some(start_point) =
                base::get_oid(matches.get_one::<String>("start_point").unwrap())
            {
                base::create_branch(name, &start_point);
                println!(
                    "Branch {} created at {}",
                    name,
                    shorten_oid(start_point.as_str())
                );
            }
        } else {
            let current = base::get_branch_name();
            for branch in base::get_branch_names() {
                if let Some(current) = &current {
                    if branch == *current {
                        println!("* {}", branch);
                        continue;
                    }
                }
                println!("{}", branch);
            }
        }
        process::exit(0);
    }

    if matches.subcommand_matches("status").is_some() {
        status();
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("reset") {
        if let Some(oid) = base::get_oid(matches.get_one::<String>("oid").unwrap()) {
            base::reset(&oid);
        }
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("diff") {
        let oid =
            base::get_oid(matches.get_one::<String>("commit").unwrap()).expect("Failed to get OID");
        diff(&oid);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("merge") {
        let oid =
            base::get_oid(matches.get_one::<String>("commit").unwrap()).expect("Failed to get OID");
        merge(&oid);
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("merge-base") {
        let commit1 = base::get_oid(matches.get_one::<String>("commit1").unwrap())
            .expect("Failed to get OID");
        let commit2 = base::get_oid(matches.get_one::<String>("commit2").unwrap())
            .expect("Failed to get OID");

        let common_ancestor = base::get_merge_base(&commit1, &commit2);

        println!("{}", common_ancestor.unwrap_or_else(|| "none".to_owned()));

        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("fetch") {
        let remote = matches.get_one::<String>("remote").unwrap();

        let mut remote_path = PathBuf::from(remote);
        remote_path.push(DEFAULT_REPO);
        ugit::remote::fetch(&remote_path);

        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("push") {
        let remote = matches.get_one::<String>("remote").unwrap();
        let branch = matches.get_one::<String>("branch").unwrap();

        let mut remote_path = PathBuf::from(remote);
        remote_path.push(DEFAULT_REPO);

        ugit::remote::push(&remote_path, branch);

        process::exit(0);
    }
}

fn print_commit(oid: &str, commit: &ugit::Commit, refs: Option<&Vec<String>>) {
    let refs_str = match refs {
        Some(refs) => format!(" ({})", refs.join(", ")),
        None => "".to_owned(),
    };
    println!("commit {}{}", oid, refs_str);
    println!("{}\n", commit.message);
}

/// Beginning at the commit with the specified OID, print the commit message and repeatedly do the
/// same for the parent commit, if it exists.
fn log(oid: &str) {
    let default_repo = &PathBuf::from(DEFAULT_REPO);

    let mut oid_to_ref: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for (ref_name, ref_value) in data::get_refs(&PathBuf::from(DEFAULT_REPO), None, true) {
        if let Some(value) = ref_value.value {
            let refs = oid_to_ref.entry(value).or_insert_with(Vec::new);
            refs.push(ref_name);
        }
    }

    for oid in base::get_commits_and_parents(default_repo, vec![oid]) {
        let commit = base::get_commit(default_repo, &oid);
        let refs = oid_to_ref.get(&oid);
        print_commit(&oid, &commit, refs);
    }
}

fn show(oid: Option<&str>) {
    let default_repo = &PathBuf::from(DEFAULT_REPO);

    if let Some(oid) = oid {
        let commit = base::get_commit(default_repo, oid);
        print_commit(oid, &commit, None);

        let parent_tree = if let Some(parent_oid) = commit.parents.first() {
            let parent_commit = base::get_commit(default_repo, parent_oid);
            Some(parent_commit.tree)
        } else {
            None
        };
        let result = diff::diff_trees(
            &base::get_tree(default_repo, parent_tree.as_deref(), None),
            &base::get_tree(default_repo, Some(commit.tree.as_str()), None),
        );
        println!("{}", result)
    }
}

fn k() {
    let mut dot = String::new();
    dot.push_str("digraph commits {\n");

    let mut ref_oids: HashSet<String> = HashSet::new();
    for (refname, ref_value) in data::get_refs(&PathBuf::from(DEFAULT_REPO), None, false) {
        dot.push_str(format!("\"{}\" [shape=note]\n", refname).as_str());
        dot.push_str(
            format!(
                "\"{}\" -> \"{}\"\n",
                refname,
                ref_value.value.clone().unwrap_or_else(|| "None".to_owned())
            )
            .as_str(),
        );
        if !ref_value.symbolic {
            if let Some(value) = ref_value.value {
                ref_oids.insert(value);
            }
        }
    }

    let default_repo = &PathBuf::from(DEFAULT_REPO);

    for oid in
        base::get_commits_and_parents(default_repo, ref_oids.iter().map(String::as_str).collect())
    {
        let commit = base::get_commit(default_repo, &oid);
        dot.push_str(
            format!(
                "\"{}\" [shape=box style=filled label=\"{}\"]\n",
                oid,
                shorten_oid(oid.as_str())
            )
            .as_str(),
        );
        if let Some(parent_oid) = commit.parents.first() {
            dot.push_str(format!("\"{}\" -> \"{}\"\n", oid, parent_oid).as_str());
        }
    }

    dot.push('}');

    println!("{}", dot);
}

fn status() {
    let head = base::get_oid("@").expect("Failed to get HEAD");
    if let Some(branch_name) = base::get_branch_name() {
        println!("On branch {}", branch_name);
    } else {
        println!("HEAD detached at {}", shorten_oid(head.as_str()));
    }

    if let Some(merge_head) = data::get_ref(&PathBuf::from(DEFAULT_REPO), "MERGE_HEAD", true).value
    {
        println!("Merging with {}", shorten_oid(&merge_head));
    }

    let default_repo = &PathBuf::from(DEFAULT_REPO);

    println!("\nChanges to be committed:\n");
    let head_tree = base::get_commit(default_repo, &head).tree;
    for (path, action) in diff::get_changed_files(
        &base::get_tree(default_repo, Some(&head_tree), None),
        &base::get_working_tree(),
    ) {
        println!("{:>12}: {}", action, path.to_str().unwrap());
    }
}

fn diff(commit: &str) {
    let default_repo = &PathBuf::from(DEFAULT_REPO);

    let tree_commit = base::get_commit(default_repo, commit);
    let tree = base::get_tree(default_repo, Some(&tree_commit.tree), None);
    let working_tree = base::get_working_tree();
    let result = diff::diff_trees(&tree, &working_tree);
    println!("{}", result);
}

fn merge(commit: &str) {
    base::merge(commit);
}

fn exit_if_not_repository() {
    let cwd = env::current_dir().expect("Failed to get current directory");
    if !base::is_ugit_repository(&cwd) {
        eprintln!("Not a ugit repository.");
        process::exit(1);
    }
}

fn shorten_oid(oid: &str) -> String {
    oid.chars().take(10).collect::<String>()
}
