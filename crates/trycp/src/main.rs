extern crate structopt;

//use log::error;
//use std::process::exit;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_ws_server::ServerBuilder;
use serde_json::{self, map::Map};
use std::{
    fs:: File,
    path::PathBuf,
    process::Command,
    io::Write,
};
use structopt::StructOpt;

type Error = String;
fn exec_output<P, S1, I, S2>(cmd: S1, args: I, dir: P, ignore_errors: bool) -> Result<String, Error>
where
    P: AsRef<std::path::Path>,
    S1: AsRef<std::ffi::OsStr>,
    I: IntoIterator<Item = S2>,
    S2: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(cmd);
    cmd.args(args)
        //        .env("N3H_VERSION_EXIT", "1")
        //        .env("NO_CLEANUP", "1")
        .current_dir(dir);
    let res = cmd
        .output()
        .map_err(|e| format!("Failed to execute {:?}: {:?}", cmd, e))?;
    if !ignore_errors && !res.status.success() {
        panic!(
            "bad exit {:?} {:?}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&res.stdout).trim().to_string())
}
#[derive(StructOpt)]
struct Cli {
    #[structopt(
        long,
        short,
        help = "The port to run the trycp server at",
        default_value = "9000"
    )]
    port: u16,
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

fn main() {
    let args = Cli::from_args();
    let mut io = IoHandler::new();
    io.add_method("player", |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let config_base64 = get_as_string("config", &params_map)?;
        let content =
            base64::decode(&config_base64).map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InvalidRequest,
                message: format!("error decoding config: {:?}", e),
                data: None,
            })?;
        let config_file = format!("src_configs/{}_src_config.toml", id);
        let file_path = PathBuf::from(config_file.clone());
        File::create(file_path)
            .map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!("unable to create config file: {:?}", e),
                data: None,
            })?
            .write_all(&content[..]).map_err(|e| jsonrpc_core::types::error::Error {
                code: jsonrpc_core::types::error::ErrorCode::InternalError,
                message: format!("unable to write config file: {:?}", e),
                data: None,
            })?;

        println!("CONTENT:{:?}", String::from_utf8_lossy(&content));
        let response = match exec_output(
            "bash",
            vec!["hcm.bash", "player", &id, &config_file],
            ".",
            true,
        ) {
            Ok(output) => output,
            Err(err) => format!("ERROR: {}", err),
        };
        println!("config {}: {:?}", id, response);
        Ok(Value::String(response))
    });
    io.add_method("spawn", |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let response = match exec_output("bash", vec!["hcm.bash", "spawn", &id], ".", true) {
            Ok(output) => output,
            Err(err) => format!("ERROR: {}", err),
        };
        println!("spawn {}: {:?}", id, response);
        Ok(Value::String(response))
    });
    io.add_method("kill", |params: Params| {
        let params_map = unwrap_params_map(params)?;
        let id = get_as_string("id", &params_map)?;
        let response = match exec_output("bash", vec!["hcm.bash", "kill", &id], ".", true) {
            Ok(output) => output,
            Err(err) => err.to_string(),
        };
        println!("kill {}: {:?}", id, response);
        Ok(Value::String(response))
    });

    let server = ServerBuilder::new(io)
        .start(&format!("0.0.0.0:{}", args.port).parse().unwrap())
        .expect("Unable to start RPC server");

    server.wait().expect("should start");
}
