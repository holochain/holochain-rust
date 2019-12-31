extern crate structopt;
extern crate url2;

mod player;

use crate::player::Sim2hCassettePlayer;
use holochain_walkman_types::Cassette;
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;

pub fn main() {
    let args = Opt::from_args();
    let file = File::open(args.path).expect("Couldn't open file for walkman");
    let cassette = Cassette::from_file(file);
    let mut player = Sim2hCassettePlayer::default();
    player.playback(cassette);
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    path: PathBuf,
}
