extern crate flame;
extern crate flamer;

use std::process::Command;
use std::fs::File;
use std::env;
use std::path::Path;

use flame as f;
use flamer::flame;


#[flame]
fn main() {
    let root = Path::new("../../");
    env::set_current_dir(&root).expect("Could not change directory to root");
    println!("Successfully changed working directory to {}!", root.display());
    Command::new("hc-app-spec-test-sim2h").spawn().expect("Could not run app-spec for this reason");
    // in order to create the flamegraph you must call one of the
    // flame::dump_* functions.
    f::dump_html(File::create("flamegraph.html").unwrap()).unwrap();
}