extern crate lib3h_protocol;
extern crate structopt;
extern crate url2;

mod player;

use crate::player::Sim2hCassettePlayer;
use holochain_walkman_types::Cassette;
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;

pub fn main() {
    let args = PlaybackSim2h::from_args();
    let sim2h_url = url2::Url2::try_parse(args.url).expect("Invalid sim2h url");
    println!("Walkman: playback from {:?} on {}", args.path, sim2h_url);
    let file = File::open(args.path).expect("Couldn't open file for walkman");
    let cassette = Cassette::from_file(file);
    Sim2hCassettePlayer::playback(&sim2h_url, cassette);
}

#[derive(StructOpt)]
struct PlaybackSim2h {
    #[structopt(short, long)]
    path: PathBuf,

    #[structopt(short, long)]
    url: String,
}
