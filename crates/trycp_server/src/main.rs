extern crate structopt;
extern crate tempfile;
#[macro_use]
extern crate serde_json;

//use log::error;
//use std::process::exit;
use self::tempfile::Builder;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_ws_server::ServerBuilder;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use regex::Regex;
use reqwest::{self, Url};
use serde_json::map::Map;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, RwLock},
};
use structopt::StructOpt;

// NOTE: don't change without also changing in crates/holochain/src/main.rs
const MAGIC_STRING: &str = "*** Done. All interfaces started.";

const CONDUCTOR_CONFIG_FILENAME: &str = "conductor-config.toml";
const CONDUCTOR_STDOUT_LOG_FILENAME: &str = "stdout.txt";
const CONDUCTOR_STDERR_LOG_FILENAME: &str = "stderr.txt";
const DNAS_DIRNAME: &str = "dnas";
const CONDUCTORS_DIRNAME: &str = "conductors";
const TRYCP_DIRNAME: &str = "/tmp/trycp";

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        long,
        short,
        help = "The port to run the trycp server on",
        default_value = "9000"
    )]
    port: u16,

    #[structopt(
        long = "port-range",
        short = "r",
        help = "The port range to use for spawning new conductors (e.g. '9000-9150'"
    )]
    port_range_string: String,

    #[structopt(long = "--allow-cmd", short = "a")]
    /// allow execution of arbitrary shell command
    allow_cmd: bool,
}

type PortRange = (u16, u16);

fn parse_port_range(s: String) -> Result<PortRange, String> {
    let segments: Vec<u16> = s
        .split('-')
        .map(|seg| {
            seg.parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())
        })
        .collect::<Result<Vec<u16>, String>>()?;
    if segments.len() == 2 {
        let (lo, hi) = (segments[0], segments[1]);
        if hi <= lo {
            Err("Port range must go from a lower port to a higher one.".into())
        } else {
            Ok((lo, hi))
        }
    } else {
        Err("Port range must be in the format 'xxxx-yyyy'".into())
    }
}

struct TrycpServer {
    // dir: tempfile::TempDir,
    dir: PathBuf,
    dna_dir: PathBuf,
    next_port: u16,
    port_range: PortRange,
}

fn make_conductor_dir() -> Result<PathBuf, String> {
    let conductor_path = PathBuf::new().join(TRYCP_DIRNAME).join(CONDUCTORS_DIRNAME);
    std::fs::create_dir_all(conductor_path.clone()).map_err(|err| format!("{:?}", err))?;
    let dir = Builder::new()
        .tempdir_in(conductor_path)
        .map_err(|err| format!("{:?}", err))?
        .into_path();
    Ok(dir)
}

fn make_dna_dir() -> Result<PathBuf, String> {
    let dna_path = PathBuf::new().join(TRYCP_DIRNAME).join(DNAS_DIRNAME);
    std::fs::create_dir_all(dna_path.clone()).map_err(|err| format!("{:?}", err))?;
    Ok(dna_path)
}

impl TrycpServer {
    pub fn new(port_range: PortRange) -> Self {
        TrycpServer {
            dir: make_conductor_dir().expect("should create conductor dir"),
            dna_dir: make_dna_dir().expect("should create dna dir"),
            next_port: port_range.0,
            port_range,
        }
    }

    pub fn acquire_port(&mut self) -> Result<u16, String> {
        let port = self.next_port;
        self.next_port += 1;
        if port >= self.port_range.1 {
            Err(format!(
                "All available ports have been used up! Range: {:?}",
                self.port_range
            ))
        } else {
            Ok(port)
        }
    }

    pub fn reset(&mut self) {
        self.next_port = self.port_range.0;
        match make_conductor_dir() {
            Err(err) => println!("reset failed creating conductor dir: {:?}", err),
            Ok(dir) => self.dir = dir,
        }
    }
}

fn unwrap_params_map(params: Params) -> Result<Map<String, Value>, jsonrpc_core::Error> {
    match params {
        Params::Map(map) => Ok(map),
        _ => Err(jsonrpc_core::Error::invalid_params("expected params map")),
    }
}

fn get_as_string<T: Into<String>>(
    key: T,
    params_map: &Map<String, Value>,
) -> Result<String, jsonrpc_core::Error> {
    let key = key.into();
    Ok(params_map
        .get(&key)
        .ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("`{}` param not provided", &key))
        })?
        .as_str()
        .ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("`{}` is not a valid json string", &key))
        })?
        .to_string())
}
fn get_as_bool<T: Into<String>>(
    key: T,
    params_map: &Map<String, Value>,
    default: Option<bool>,
) -> Result<bool, jsonrpc_core::Error> {
    let key = key.into();
    match params_map.get(&key) {
        Some(value) => value.as_bool().ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("`{}` has to be a boolean", &key))
        }),
        None => default.ok_or_else(|| {
            jsonrpc_core::Error::invalid_params(format!("required param `{}` not provided", &key))
        }),
    }
}

fn get_dir(state: &TrycpServer, id: &String) -> PathBuf {
    state.dir.join(id)
}

fn get_config_path(state: &TrycpServer, id: &String) -> PathBuf {
    get_dir(state, id).join(CONDUCTOR_CONFIG_FILENAME)
}

fn get_dna_dir(state: &TrycpServer) -> PathBuf {
    state.dna_dir.clone()
}

fn get_dna_path(state: &TrycpServer, url: &Url) -> PathBuf {
    get_dna_dir(state).join(url.path().to_string().replace("/", "").replace("%", "_"))
}

fn get_stdout_log_path(state: &TrycpServer, id: &String) -> PathBuf {
    get_dir(state, id).join(CONDUCTOR_STDOUT_LOG_FILENAME)
}

fn get_stderr_log_path(state: &TrycpServer, id: &String) -> PathBuf {
    get_dir(state, id).join(CONDUCTOR_STDERR_LOG_FILENAME)
}

fn internal_error(message: String) -> jsonrpc_core::types::error::Error {
    jsonrpc_core::types::error::Error {
        code: jsonrpc_core::types::error::ErrorCode::InternalError,
        message,
        data: None,
    }
}

fn invalid_request(message: String) -> jsonrpc_core::types::error::Error {
    jsonrpc_core::types::error::Error {
        code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
        message,
        data: None,
    }
}

fn save_file(file_path: PathBuf, content: &[u8]) -> Result<(), jsonrpc_core::types::error::Error> {
    File::create(file_path.clone())
        .map_err(|e| {
            internal_error(format!(
                "unable to create file: {:?} {}",
                e,
                file_path.to_string_lossy()
            ))
        })?
        .write_all(&content[..])
        .map_err(|e| {
            internal_error(format!(
                "unable to write file: {:?} {}",
                e,
                file_path.to_string_lossy()
            ))
        })?;
    Ok(())
}

fn get_info_as_json() -> String {
    let output = Command::new("holochain")
        .args(&["-i"])
        .output()
        .expect("failed to execute process");
    let info_str = String::from_utf8(output.stdout).unwrap();

    // poor mans JSON convert
    let re = Regex::new(r"(?P<key>[^:]+):\s+(?P<val>.*)\n").unwrap();
    let result = re.replace_all(&info_str, "\"$key\": \"$val\",");
    let mut result = format!("{}", result); // pop off the final comma
    result.pop();
    format!("{{{}}}", result)
}

/// very dangerous, runs whatever strings come in from the internet directly in bash
fn os_eval(arbitrary_command: &str) -> String {
    println!("running cmd {}", arbitrary_command);
    match Command::new("bash")
        .args(&["-c", arbitrary_command])
        .output()
    {
        Ok(output) => {
            let response = if output.status.success() {
                &output.stdout
            } else {
                &output.stderr
            };
            String::from_utf8_lossy(response).trim_end().to_string()
        }
        Err(err) => format!("cmd err: {:?}", err),
    }
}

fn main() {
    let args = Cli::from_args();
    let mut io = IoHandler::new();

    let conductor_port_range: PortRange =
        parse_port_range(args.port_range_string).expect("Invalid port range");

    let state: Arc<RwLock<TrycpServer>> =
        Arc::new(RwLock::new(TrycpServer::new(conductor_port_range)));
    let state_setup = state.clone();
    let state_player = state.clone();
    let state_spawn = state.clone();
    let state_kill = state.clone();
    let state_dna = state.clone();

    let players_arc: Arc<RwLock<HashMap<String, Child>>> = Arc::new(RwLock::new(HashMap::new()));
    let players_arc_kill = players_arc.clone();
    let players_arc_reset = players_arc.clone();
    let players_arc_spawn = players_arc;

    io.add_method("ping", |_params: Params| {
        Ok(Value::String(get_info_as_json()))
    });
    let allow_cmd = args.allow_cmd;
    io.add_method("cmd", move |params: Params| {
        if allow_cmd {
            Ok(Value::String(os_eval(&get_as_string(
                "cmd",
                &unwrap_params_map(params)?,
            )?)))
        } else {
            println!("cmd command not allowed");
            Ok(Value::String("cmd not allowed".to_string()))
        }
    });

    io.add_method("dna", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let url_str = get_as_string("url", &params_map)?;
        let url = Url::parse(&url_str).map_err(|e| {
            invalid_request(format!("unable to parse url:{} got error: {}", url_str, e))
        })?;
        let state = state_dna.write().unwrap();
        let file_path = get_dna_path(&state, &url);
        if !file_path.exists() {
            println!("Downloading dna from {} ...", &url_str);
            let content: String = reqwest::get::<Url>(url)
                .map_err(|e| {
                    internal_error(format!("error downloading dna: {:?} {:?}", e, url_str))
                })?
                .text()
                .map_err(|e| internal_error(format!("could not get text response: {}", e)))?;
            println!("Finished downloading dna from {}", url_str);
            let dir_path = get_dna_dir(&state);
            std::fs::create_dir_all(dir_path.clone()).map_err(|e| {
                internal_error(format!(
                    "error making temporary directory for dna: {:?} {:?}",
                    e, dir_path
                ))
            })?;
            save_file(file_path.clone(), &content.as_bytes())?;
        }
        let local_path = file_path.to_string_lossy();
        let response = format!("dna for {} at {}", &url_str, local_path,);
        println!("dna {}: {:?}", &url_str, response);
        Ok(json!({ "path": local_path }))
    });

    io.add_method("reset", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let killall = get_as_bool("killall", &params_map, Some(false))?;
        {
            let mut players = players_arc_reset.write().expect("should_lock");
            if killall {
                let output = Command::new("killall")
                    .args(&["holochain", "-s", "SIGKILL"])
                    .output()
                    .expect("failed to execute process");
                println!("killall result: {:?}", output);
            } else {
                for (id, child) in &*players {
                    let _ = do_kill(id, child, "SIGKILL"); //ignore any errors
                }
            }
            players.clear();
        }
        {
            let mut temp_path = state.write().expect("should_lock");
            temp_path.reset();
        }

        Ok(Value::String("reset".into()))
    });

    // Return to try-o-rama information it can use to build config files
    // i.e. ensure ports are open, and ensure that configDir is the same one
    // that the actual config will be written to
    io.add_method("setup", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let mut state = state_setup.write().unwrap();
        let file_path = get_dir(&state, &id);
        let interface_port = state.acquire_port().map_err(|e| internal_error(e))?;
        Ok(json!({
            "interfacePort": interface_port,
            "configDir": file_path.to_string_lossy(),
        }))
    });

    io.add_method("player", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let config_base64 = get_as_string("config", &params_map)?;
        let content = base64::decode(&config_base64)
            .map_err(|e| invalid_request(format!("error decoding config: {:?}", e)))?;
        let file_path = {
            let state = state_player.read().unwrap();
            let dir_path = get_dir(&state, &id);
            std::fs::create_dir_all(dir_path.clone()).map_err(|e| {
                invalid_request(format!(
                    "error making temporary directory for config: {:?} {:?}",
                    e, dir_path
                ))
            })?;
            get_config_path(&state, &id)
        };
        save_file(file_path.clone(), &content)?;
        let response = format!(
            "wrote config for player {} to {}",
            id,
            file_path.to_string_lossy()
        );
        println!("player {}: {:?}", id, response);
        Ok(Value::String(response))
    });

    io.add_method("spawn", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;

        let state = state_spawn.read().unwrap();
        check_player_config(&state, &id)?;
        let mut players = players_arc_spawn.write().expect("should_lock");
        if players.contains_key(&id) {
            return Err(invalid_request(format!("{} is already running", id)));
        };

        let config_path = get_config_path(&state, &id).to_str().unwrap().to_string();
        let stdout_log_path = get_stdout_log_path(&state, &id)
            .to_str()
            .unwrap()
            .to_string();
        let stderr_log_path = get_stderr_log_path(&state, &id)
            .to_str()
            .unwrap()
            .to_string();

        let mut conductor = Command::new("holochain")
            .args(&["-c", &config_path])
            .env("RUST_BACKTRACE", "full")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| internal_error(format!("unable to spawn conductor: {:?}", e)))?;

        let mut log_stdout = Command::new("tee")
            .arg(stdout_log_path)
            .stdout(Stdio::piped())
            .stdin(conductor.stdout.take().unwrap())
            .spawn()
            .unwrap();

        let _log_stderr = Command::new("tee")
            .arg(stderr_log_path)
            .stdin(conductor.stderr.take().unwrap())
            .spawn()
            .unwrap();

        match log_stdout.stdout.take() {
            Some(stdout) => {
                for line in BufReader::new(stdout).lines() {
                    let line = line.unwrap();
                    if line == MAGIC_STRING {
                        println!("Encountered magic string");
                        break;
                    }
                }

                players.insert(id.clone(), conductor);
                let response = format!("conductor spawned for {}", id);
                Ok(Value::String(response))
            }
            None => {
                conductor.kill().unwrap();
                Err(internal_error(
                    "Conductor process not capturing stdout, bailing!".to_string(),
                ))
            }
        }
    });

    io.add_method("kill", move |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let signal = get_as_string("signal", &params_map)?; // TODO: make optional?

        check_player_config(&state_kill.read().unwrap(), &id)?;
        let mut players = players_arc_kill.write().unwrap();
        match players.remove(&id) {
            None => {
                return Err(invalid_request(format!("no conductor spawned for {}", id)));
            }
            Some(ref mut child) => {
                do_kill(&id, child, signal.as_str())?;
            }
        }
        let response = format!("killed conductor for {}", id);
        Ok(Value::String(response))
    });

    let server = ServerBuilder::new(io)
        .start(&format!("0.0.0.0:{}", args.port).parse().unwrap())
        .expect("server should start");

    println!("waiting for connections on port {}", args.port);

    server.wait().expect("server should wait");
}

fn do_kill(
    id: &String,
    child: &Child,
    signal: &str,
) -> Result<(), jsonrpc_core::types::error::Error> {
    let sig = match signal {
        "SIGKILL" => Signal::SIGKILL,
        "SIGTERM" => Signal::SIGTERM,
        _ => Signal::SIGINT,
    };
    signal::kill(Pid::from_raw(child.id() as i32), sig).map_err(|e| {
        internal_error(format!(
            "unable to run kill conductor for {} script: {:?}",
            id, e
        ))
    })
}

fn check_player_config(
    state: &TrycpServer,
    id: &String,
) -> Result<(), jsonrpc_core::types::error::Error> {
    let file_path = get_config_path(state, id);
    if !file_path.is_file() {
        return Err(invalid_request(format!(
            "player config for {} not setup",
            id
        )));
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {

    use crate::os_eval;

    #[test]
    fn os_eval_test() {
        assert_eq!("foo", os_eval("echo foo"));
        assert_eq!("bash: zzz: command not found", os_eval("zzz yyy"));
    }
}
