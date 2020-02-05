mod player;

use crate::player::{deserialize_message_data, get_wire_message, Sim2hCassettePlayer};
use holochain_walkman_types::{Cassette, WalkmanEvent, WalkmanSim2hEvent};
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;

pub fn main() {
    match Opt::from_args() {
        Opt::Cassette(opt_cassette) => match opt_cassette {
            OptCassette::Compile(OptPath { path }) => {
                let cassette = read_cassette_from_log(path);
                let stdout = std::io::stdout();
                let wtr = snap::Writer::new(stdout.lock());
                serde_json::to_writer(wtr, &cassette).expect("Could not serialize cassette data");
            }
            OptCassette::Show(OptPath { path }) => {
                let cassette = read_cassette(path);
                for item in cassette.logs() {
                    let time: chrono::DateTime<chrono::offset::Local> = item.time.into();
                    let WalkmanEvent::Sim2hEvent(event) = &item.event;
                    let line = match event {
                        WalkmanSim2hEvent::Connect(url) => format!("{} CONNECT", url),
                        WalkmanSim2hEvent::Disconnect(url) => format!("{} DISCONNECT", url),
                        WalkmanSim2hEvent::Message(url, data) => {
                            let signed = deserialize_message_data(&data);
                            let wire_msg = get_wire_message(&signed);
                            format!("{} MSG : {:?}", url, wire_msg)
                        }
                    };
                    println!("{:?} {}", time.timestamp(), line);
                }
            }
            OptCassette::Raw(OptPath { path }) => {
                let cassette = read_cassette(path);
                let stdout = std::io::stdout();
                serde_json::to_writer(stdout, &cassette)
                    .expect("Could not serialize cassette data");
            }
            OptCassette::CompileRaw(OptPath { path }) => {
                let cassette = read_raw(path);
                let stdout = std::io::stdout();
                let wtr = snap::Writer::new(stdout.lock());
                serde_json::to_writer(wtr, &cassette).expect("Could not serialize cassette data");
            }
        },
        Opt::Playback(opt_sim2h) => match opt_sim2h {
            OptPlayback::Sim2h(playback) => {
                let sim2h_url = url2::Url2::try_parse(playback.url).expect("Invalid sim2h url");
                println!(
                    "Walkman: playback from {:?} on {}",
                    playback.path, sim2h_url
                );
                let cassette = if playback.raw {
                    read_cassette_from_log(playback.path)
                } else {
                    read_cassette(playback.path)
                };
                Sim2hCassettePlayer::new().playback(&sim2h_url, cassette);
            }
        },
    }
}

fn read_cassette(path: PathBuf) -> Cassette {
    let file = File::open(path).expect("Couldn't open file for walkman");
    let reader = std::io::BufReader::new(file);
    let reader = snap::Reader::new(reader);
    serde_json::from_reader(reader).expect("Invalid cassette file")
}

fn read_raw(path: PathBuf) -> Cassette {
    let file = File::open(path).expect("Couldn't open file for walkman");
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).expect("Invalid cassette file")
}

fn read_cassette_from_log(path: PathBuf) -> Cassette {
    let file = File::open(path).expect("Couldn't open file for walkman");
    let reader = std::io::BufReader::new(file);
    Cassette::from_log_data(reader)
}

#[derive(StructOpt)]
enum Opt {
    Cassette(OptCassette),
    Playback(OptPlayback),
}

#[derive(StructOpt)]
enum OptCassette {
    Compile(OptCompile),
    Show(OptShow),
    Raw(OptRaw),
    CompileRaw(OptCompileRaw),
}

#[derive(StructOpt)]
enum OptPlayback {
    Sim2h(OptSim2hPlayback),
}

#[derive(StructOpt)]
struct OptPath {
    #[structopt(short, long)]
    path: PathBuf,
}

type OptCompile = OptPath;
type OptShow = OptPath;
type OptRaw = OptPath;
type OptCompileRaw = OptPath;

#[derive(StructOpt)]
struct OptSim2hPlayback {
    #[structopt(short, long)]
    path: PathBuf,

    #[structopt(short, long)]
    url: String,

    #[structopt(short, long)]
    raw: bool,
}
