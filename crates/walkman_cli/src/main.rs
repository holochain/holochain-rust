extern crate lib3h_protocol;
extern crate structopt;
extern crate url2;

mod player;

use crate::player::Sim2hCassettePlayer;
use holochain_walkman_types::Cassette;
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;

pub fn main() {
    match Opt::from_args() {
        Opt::Sim2h(opt_sim2h) => match opt_sim2h {
            OptSim2h::Compile(OptSim2hCompile {path}) => {
                let file = File::open(path).expect("Couldn't open file for walkman");
                let reader = std::io::BufReader::new(file);
                let cassette = Cassette::from_log_data(reader);
                println!("{}", serde_json::to_string(&cassette).expect("Could not serialize cassette data"));
            }
            OptSim2h::Playback(playback) => {
                let sim2h_url = url2::Url2::try_parse(playback.url).expect("Invalid sim2h url");
                println!("Walkman: playback from {:?} on {}", playback.path, sim2h_url);
                let file = File::open(playback.path).expect("Couldn't open file for walkman");
                let reader = std::io::BufReader::new(file);
                let cassette = if playback.compile {
                    Cassette::from_log_data(reader)
                } else {
                    serde_json::from_reader(reader).expect("Invalid cassette file")
                };
                Sim2hCassettePlayer::playback(&sim2h_url, cassette);
            }
        }
    }
}


#[derive(StructOpt)]
enum Opt {
    Sim2h(OptSim2h)
}

#[derive(StructOpt)]
enum OptSim2h {
    Playback(OptSim2hPlayback),
    Compile(OptSim2hCompile),
}

#[derive(StructOpt)]
struct OptSim2hCompile {
    #[structopt(short, long)]
    path: PathBuf,
}

#[derive(StructOpt)]
struct OptSim2hPlayback {
    #[structopt(short, long)]
    path: PathBuf,

    #[structopt(short, long)]
    url: String,

    #[structopt(short, long)]
    compile: bool,
}
