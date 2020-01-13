use crate::websocket::{mem_stream::*, streams::*, tls::TlsConfig, wss_info::WssInfo};
use lib3h::transport::error::*;

use url2::prelude::*;

#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl StreamManager<MemStream> {
    pub fn with_mem_stream(tls_config: TlsConfig) -> Self {
        let bind: Bind<MemStream> = Box::new(move |url| Self::mem_bind(&Url2::from(url)));
        StreamManager::new(
            |uri| Ok(MemStream::connect(&Url2::parse(uri))?),
            bind,
            tls_config,
        )
    }

    fn mem_bind(url: &Url2) -> TransportResult<(Url2, Acceptor<MemStream>)> {
        let mut listener = MemListener::bind(&url)?;
        let url = listener.get_url().clone();
        Ok((
            url,
            Box::new(move || match listener.accept() {
                Ok(stream) => Ok(WssInfo::server(stream.get_url().clone().into(), stream)),
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    Err(TransportError::new_kind(ErrorKind::Ignore(err.to_string())))
                }
                Err(e) => Err(e.into()),
            }),
        ))
    }
}
