extern crate structopt;

use structopt::StructOpt;
use std::{fs::File, path::PathBuf};
use holochain_walkman_types::{Cassette};

pub fn main() {
    let args = Opt::from_args();
    let file = File::open(args.path).expect("Couldn't open file for walkman");
    let cassette = Cassette::from_file(file);

}

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    path: PathBuf
}
