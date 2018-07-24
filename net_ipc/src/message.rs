use serde_bytes;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCliSend<'a> (
    #[serde(with = "serde_bytes")]
    pub &'a[u8],
    #[serde(with = "serde_bytes")]
    pub &'a[u8],
    #[serde(with = "serde_bytes")]
    pub &'a[u8]
);

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MsgCliCall<'a> (
    #[serde(with = "serde_bytes")]
    pub &'a[u8],
    #[serde(with = "serde_bytes")]
    pub &'a[u8],
    #[serde(with = "serde_bytes")]
    pub &'a[u8],
    #[serde(with = "serde_bytes")]
    pub &'a[u8]
);

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvPong (pub f64, pub f64);

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRespOk (
    #[serde(with = "serde_bytes")]
    pub Vec<u8>
);

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRecvSend (
    #[serde(with = "serde_bytes")]
    pub Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub Vec<u8>
);

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRecvCall (
    #[serde(with = "serde_bytes")]
    pub Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub Vec<u8>
);

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MsgSrvRecvCallResp (
    #[serde(with = "serde_bytes")]
    pub Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub Vec<u8>
);

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    SrvPong(MsgSrvPong),
    SrvRespOk(MsgSrvRespOk),
    SrvRecvSend(MsgSrvRecvSend),
    SrvRecvCall(MsgSrvRecvCall),
    SrvRecvCallResp(MsgSrvRecvCallResp),
}
