use std::{
    env, fs,
    io::{self, Write},
    process,
};

use clap::{App, Arg, SubCommand};

mod ugit;

fn main() {
    let matches = App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .subcommand(SubCommand::with_name("init").about("Create a new ugit repository"))
        .subcommand(
            SubCommand::with_name("hash-object")
                .about("Add the specified file to the object store and print its OID")
                .arg(Arg::with_name("filename").required(true)),
        )
        .subcommand(
            SubCommand::with_name("cat-file")
                .about("Print the contents of the file with the specified OID")
                .arg(Arg::with_name("oid").required(true)),
        )
        .subcommand(
            SubCommand::with_name("write-tree")
                .about("Write the current directory to the object store"),
        )
        .subcommand(
            SubCommand::with_name("read-tree").about("Replace the contents of the current directory with the tree with the specified OID").arg(Arg::with_name("tree_oid").required(true)),
        )
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        ugit::data::init();
        process::exit(0);
    }

    // All of the subsequent subcommands need to be run within an existing ugit repository, so exit
    // with an error if the current working directory isn't one.
    exit_if_not_repository();

    if let Some(matches) = matches.subcommand_matches("hash-object") {
        let filename = matches.value_of("filename").unwrap();
        let contents = fs::read(filename).expect("Failed to read file contents");
        let object_hash = ugit::data::hash_object(&contents, "blob");
        io::stdout()
            .write_all(object_hash.as_bytes())
            .expect("Failed to output object ID");
        io::stdout().write(b"\n").expect("Failed to output newline");
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("cat-file") {
        let oid = matches.value_of("oid").unwrap();
        let contents = ugit::data::get_object(oid, None);
        io::stdout()
            .write_all(&contents)
            .expect("Failed to output file data");
        process::exit(0);
    }

    if let Some(_matches) = matches.subcommand_matches("write-tree") {
        let cwd = env::current_dir().expect("Failed to get current working directory");
        ugit::base::write_tree(cwd.as_path());
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("read-tree") {
        let tree_oid = matches.value_of("tree_oid").unwrap();
        ugit::base::read_tree(tree_oid);
        process::exit(0);
    }
}

fn exit_if_not_repository() {
    let cwd = env::current_dir().expect("Failed to get current directory");
    if !ugit::base::is_ugit_repository(&cwd) {
        eprintln!("Not a ugit repository.");
        process::exit(1);
    }
}
