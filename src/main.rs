#[macro_use]
extern crate clap;
extern crate map18xx;

use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("map18xx")
        .version(crate_version!())
        .author(crate_authors!())
        .about("18xx tile and map designer")
        .arg(Arg::with_name("verbose")
             .help("Print debug information")
             .short("v")
             .long("verbose")
             .global(true))
        .arg(Arg::with_name("debug_coordinates")
             .help("Show coordinates on each row/column")
             .short("c")
             .long("debug_coordinates")
             .global(true))
        .subcommand(SubCommand::with_name("asset")
                    .about("Generate assets to PnP game")
                    .aliases(&["a", "assets"])
                    .arg(Arg::with_name("game")
                         .help("Game for which to generate assets")
                         .required(true)
                         .index(1)))
        .get_matches();

    let mut options = map18xx::Options::new();
    options.verbose = matches.is_present("verbose");
    options.debug_coordinates = matches.is_present("debug_coordinates");

    // Determine subcommand
    match matches.subcommand() {
        ("asset", Some(ref matches)) => {
            let mut asset_options = map18xx::AssetOptions::new();
            asset_options.name = matches.value_of("game").unwrap().to_string();
            map18xx::asset_mode(&options, &asset_options);
        }
        ("", _) => map18xx::definitions(&options),
        (name, _) => eprintln!("Unkown subcommand {}.", name),
    }
}
