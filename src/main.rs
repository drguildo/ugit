use clap::App;

mod ugit;

fn main() {
    let matches = App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .subcommand(clap::SubCommand::with_name("init"))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("init") {
        ugit::data::init();
    }
}
