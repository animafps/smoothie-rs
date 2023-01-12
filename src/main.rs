// #[macro_use]
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use clap::Parser;

mod cli;
mod recipe;
// mod recipe;
// mod video;
// mod exec;

fn main() {
    color_eyre::install().unwrap();

    let _args = cli::Arguments::parse();

    // let rc: HashMap <String, String> = HashMap::new();
    let _rc = recipe::get_recipe(&_args);
    // let _videos = video::resolve_input(rc, _args);
    // exec::smoothing(videos);
}
