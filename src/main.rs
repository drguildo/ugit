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
        .subcommand(SubCommand::with_name("init"))
        .subcommand(
            SubCommand::with_name("hash-object").arg(Arg::with_name("filename").required(true)),
        )
        .subcommand(SubCommand::with_name("cat-file").arg(Arg::with_name("oid").required(true)))
        .subcommand(SubCommand::with_name("write-tree"))
        .subcommand(
            SubCommand::with_name("read-tree").arg(Arg::with_name("tree_oid").required(true)),
        )
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        ugit::data::init();
        process::exit(0);
    }

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
