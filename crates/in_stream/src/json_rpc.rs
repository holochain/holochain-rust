use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: String,
}

impl JsonRpcRequest {
    pub fn new(id: &str, method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: String,
}

impl JsonRpcResponse {
    pub fn new_result(id: &str, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: id.to_string(),
        }
    }

    pub fn new_error(id: &str, code: i64, message: &str, data: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data,
            }),
            id: id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::*;

    #[test]
    fn json_rpc_serialize() {
        let msg = JsonRpcRequest::new("42", "testing", json!([1, 2]));
        println!("{:?}", msg);
        println!("{}", serde_json::to_string_pretty(&msg).unwrap());
        let msg = JsonRpcResponse::new_result("42", json!("hello"));
        println!("{:?}", msg);
        println!("{}", serde_json::to_string_pretty(&msg).unwrap());
        let msg = JsonRpcResponse::new_error("42", 42, "test-error", Some(json!("bad")));
        println!("{:?}", msg);
        println!("{}", serde_json::to_string_pretty(&msg).unwrap());
    }

    fn wait_read<Sub: 'static + InStreamStd>(s: &mut InStreamWss<Sub>) -> WsFrame {
        let mut out = WsFrame::default();
        loop {
            match s.read(&mut out) {
                Ok(_) => return out,
                Err(e) if e.would_block() => std::thread::yield_now(),
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    #[test]
    fn json_rpc_suite() {
        let mut url = in_stream_mem::random_url("test");
        url.set_scheme("ws").unwrap();
        let config = MemBindConfig::default();
        let config = WssBindConfig::new(config);
        let mut l: InStreamListenerWss<InStreamListenerMem> =
            InStreamListenerWss::bind(&url, config).unwrap();
        let binding = l.binding();
        let thread = std::thread::spawn(move || {
            let mut srv = loop {
                match l.accept() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => std::thread::yield_now(),
                    Err(e) => panic!("{:?}", e),
                }
            };

            println!("GOT CONNECTION {}", srv.remote_url());

            let req = wait_read(&mut srv);
            let req: JsonRpcRequest = serde_json::from_slice(req.as_bytes()).unwrap();
            println!("got request: {:?}", req);

            srv.write(
                serde_json::to_vec(&JsonRpcResponse::new_result(
                    "42",
                    json!({"got_request_params": req.params}),
                ))
                .unwrap()
                .into(),
            )
            .unwrap();
            srv.flush().unwrap();
        });

        let mut cli: InStreamWss<InStreamMem> =
            InStreamWss::connect(&binding, WssConnectConfig::new(MemConnectConfig::default()))
                .unwrap();

        cli.write(
            serde_json::to_vec(&JsonRpcRequest::new(
                "42",
                "testing",
                json!([1, 2, "chickens"]),
            ))
            .unwrap()
            .into(),
        )
        .unwrap();
        cli.flush().unwrap();

        let res = wait_read(&mut cli);
        let res: JsonRpcResponse = serde_json::from_slice(res.as_bytes()).unwrap();
        println!("got response: {:?}", res);

        thread.join().unwrap();

        println!("done");
    }
}
