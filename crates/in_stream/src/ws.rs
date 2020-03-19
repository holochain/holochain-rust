use crate::*;
use std::io::{Error, ErrorKind, Result};
use url2::prelude::*;

mod frame;
pub use frame::*;

const SCHEME: &'static str = "ws";

/// internal helper, make sure we're dealing with wss urls
fn validate_url_scheme(url: &Url2) -> Result<()> {
    if url.scheme() != SCHEME {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("got: '{}', expected: '{}://...'", SCHEME, url),
        ));
    }
    Ok(())
}

#[derive(Debug)]
/// websocket specific bind configuration
pub struct WssBindConfig {
    pub sub_bind_config: InStreamConfigAny,
    pub disconnect_on_slow_pong_ms: Option<u64>,
}

impl WssBindConfig {
    pub fn new<Sub: InStreamConfig>(sub_config: Sub) -> Self {
        Self {
            sub_bind_config: sub_config.to_any(),
            disconnect_on_slow_pong_ms: None,
        }
    }

    pub fn disconnect_on_slow_pong_ms(mut self, ms: Option<u64>) -> Self {
        self.disconnect_on_slow_pong_ms = ms;
        self
    }
}

impl InStreamConfig for WssBindConfig {}

/// bind to a network interface to listen for websocket connections
#[derive(Debug)]
pub struct InStreamListenerWss<Sub: InStreamListenerStd> {
    sub: Sub,
    disconnect_on_slow_pong_ms: Option<u64>,
}

impl<Sub: InStreamListenerStd> InStreamListenerWss<Sub> {
    pub fn bind(url: &Url2, config: WssBindConfig) -> Result<Self> {
        InStreamListenerWss::raw_bind(url, config)
    }
}

impl<Sub: InStreamListenerStd> InStreamListener<&mut WsFrame, WsFrame>
    for InStreamListenerWss<Sub>
{
    type Stream = InStreamWss<Sub::StreamStd>;

    fn raw_bind<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let config = WssBindConfig::from_gen(config)?;
        validate_url_scheme(url)?;
        let mut url = url.clone();
        // will only fail if scheme is mal-formed, but it's a constant
        // so unwrap() is Ok
        url.set_scheme(Sub::StreamStd::URL_SCHEME).unwrap();
        let sub = Sub::raw_bind(&url, config.sub_bind_config)?;
        Ok(Self {
            sub,
            disconnect_on_slow_pong_ms: config.disconnect_on_slow_pong_ms,
        })
    }

    fn binding(&self) -> Url2 {
        let mut url = self.sub.binding();
        url.set_scheme(SCHEME).unwrap();
        url
    }

    fn accept(&mut self) -> Result<<Self as InStreamListener<&mut WsFrame, WsFrame>>::Stream> {
        let stream: Sub::StreamStd = self.sub.accept_std()?;

        let s = stream.into_std_stream();
        log::debug!("ws: calling accept on {:?}", s);
        let res = tungstenite::accept(s);
        let mut out = InStreamWss::priv_new(Url2::default(), self.disconnect_on_slow_pong_ms);
        match out.priv_proc_wss_srv_result(res) {
            Ok(_) => Ok(out),
            Err(e) if e.would_block() => Ok(out),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
/// websocket specific connect config
pub struct WssConnectConfig {
    pub sub_connect_config: InStreamConfigAny,
    pub disconnect_on_slow_pong_ms: Option<u64>,
}

impl WssConnectConfig {
    pub fn new<Sub: InStreamConfig>(sub_config: Sub) -> Self {
        Self {
            sub_connect_config: sub_config.to_any(),
            disconnect_on_slow_pong_ms: None,
        }
    }

    pub fn disconnect_on_slow_pong_ms(mut self, ms: Option<u64>) -> Self {
        self.disconnect_on_slow_pong_ms = ms;
        self
    }
}

impl InStreamConfig for WssConnectConfig {}

#[derive(Debug)]
enum WssState<Sub: InStreamStd> {
    MidCliHandshake(
        tungstenite::handshake::MidHandshake<tungstenite::ClientHandshake<StdStreamAdapter<Sub>>>,
    ),
    MidSrvHandshake(
        tungstenite::handshake::MidHandshake<
            tungstenite::ServerHandshake<
                StdStreamAdapter<Sub>,
                tungstenite::handshake::server::NoCallback,
            >,
        >,
    ),
    Ready(tungstenite::WebSocket<StdStreamAdapter<Sub>>),
}

/// websocket stream
#[derive(Debug)]
pub struct InStreamWss<Sub: InStreamStd> {
    state: Option<WssState<Sub>>,
    remote_url: Url2,
    write_buf: std::collections::VecDeque<WsFrame>,
    last_ping: std::time::Instant,
    last_pong: std::time::Instant,
    disconnect_on_slow_pong_ms: Option<u64>,
}

type TungsteniteCliHandshakeResult<S> = std::result::Result<
    (
        tungstenite::WebSocket<S>,
        tungstenite::handshake::client::Response,
    ),
    tungstenite::handshake::HandshakeError<tungstenite::handshake::client::ClientHandshake<S>>,
>;

type TungsteniteSrvHandshakeResult<S> = std::result::Result<
    tungstenite::WebSocket<S>,
    tungstenite::handshake::HandshakeError<
        tungstenite::handshake::server::ServerHandshake<
            S,
            tungstenite::handshake::server::NoCallback,
        >,
    >,
>;

fn priv_write_wss_frame<S: InStreamStd>(
    wss: &mut tungstenite::WebSocket<StdStreamAdapter<S>>,
    frame: WsFrame,
) -> Result<()> {
    let res = wss.write_message(frame.into());
    match res {
        Ok(_) => Ok(()),
        // ignore would-block errors on write
        // tungstenite queues them in pending, they'll get sent
        Err(tungstenite::error::Error::Io(e)) if e.would_block() => Ok(()),
        Err(tungstenite::error::Error::Io(_)) => {
            if let Err(tungstenite::error::Error::Io(e)) = res {
                Err(e)
            } else {
                unreachable!();
            }
        }
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("tungstenite error: {:?}", e),
        )),
    }
}

impl<Sub: InStreamStd> InStreamWss<Sub> {
    pub fn connect(url: &Url2, config: WssConnectConfig) -> Result<Self> {
        InStreamWss::raw_connect(url, config)
    }

    fn priv_new(remote_url: Url2, disconnect_on_slow_pong_ms: Option<u64>) -> Self {
        Self {
            state: None,
            remote_url,
            write_buf: std::collections::VecDeque::new(),
            last_ping: std::time::Instant::now(),
            last_pong: std::time::Instant::now(),
            disconnect_on_slow_pong_ms,
        }
    }

    fn priv_proc_wss_cli_result(
        &mut self,
        result: TungsteniteCliHandshakeResult<StdStreamAdapter<Sub>>,
    ) -> Result<()> {
        match result {
            Ok((stream, _response)) => {
                self.state = Some(WssState::Ready(stream));
                self.last_ping = std::time::Instant::now();
                self.last_pong = std::time::Instant::now();
                self.priv_write_pending()?;
                Ok(())
            }
            Err(tungstenite::HandshakeError::Interrupted(mid)) => {
                self.state = Some(WssState::MidCliHandshake(mid));
                Err(Error::with_would_block())
            }
            Err(e) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!("wss_cli: {:?}", e),
            )),
        }
    }

    fn priv_proc_wss_srv_result(
        &mut self,
        result: TungsteniteSrvHandshakeResult<StdStreamAdapter<Sub>>,
    ) -> Result<()> {
        match result {
            Ok(stream) => {
                self.remote_url = stream.get_ref().remote_url();
                self.remote_url.set_scheme(SCHEME).unwrap();
                self.state = Some(WssState::Ready(stream));
                self.last_ping = std::time::Instant::now();
                self.last_pong = std::time::Instant::now();
                self.priv_write_pending()?;
                Ok(())
            }
            Err(tungstenite::HandshakeError::Interrupted(mid)) => {
                self.remote_url = mid.get_ref().get_ref().remote_url();
                self.remote_url.set_scheme(SCHEME).unwrap();
                self.state = Some(WssState::MidSrvHandshake(mid));
                Err(Error::with_would_block())
            }
            Err(e) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!("ws_srv: {:?}", e),
            )),
        }
    }

    fn priv_process(&mut self) -> Result<()> {
        if self.state.is_none() {
            return Ok(());
        }

        if let WssState::Ready(wss) = self.state.as_mut().unwrap() {
            if let Some(pong_ms) = self.disconnect_on_slow_pong_ms {
                // send out pings twice as frequently as we expect pongs
                let ping_ms = pong_ms / 2;
                if self.last_ping.elapsed().as_millis() as u64 > ping_ms {
                    // skip our send queue
                    priv_write_wss_frame(wss, WsFrame::Ping(Vec::with_capacity(0)))?;
                    self.last_ping = std::time::Instant::now();
                }
            }
            return Ok(());
        }

        match match self.state.take().unwrap() {
            WssState::MidCliHandshake(mid) => self.priv_proc_wss_cli_result(mid.handshake()),
            WssState::MidSrvHandshake(mid) => self.priv_proc_wss_srv_result(mid.handshake()),
            _ => unreachable!(),
        } {
            Ok(_) => Ok(()),
            Err(e) if e.would_block() => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn priv_write_pending(&mut self) -> Result<()> {
        loop {
            if self.write_buf.is_empty() {
                return Ok(());
            }
            match &mut self.state {
                None => return Err(ErrorKind::NotConnected.into()),
                Some(state) => {
                    if let WssState::Ready(wss) = state {
                        priv_write_wss_frame(wss, self.write_buf.pop_front().unwrap())?;
                    } else {
                        return Ok(());
                    }
                }
            }
        }
    }
}

impl<Sub: InStreamStd> InStream<&mut WsFrame, WsFrame> for InStreamWss<Sub> {
    const URL_SCHEME: &'static str = SCHEME;

    fn raw_connect<C: InStreamConfig>(url: &Url2, config: C) -> Result<Self> {
        let config = WssConnectConfig::from_gen(config)?;
        validate_url_scheme(url)?;
        let remote_url = url.clone();
        let mut url = url.clone();
        url.set_scheme(Sub::URL_SCHEME).unwrap();
        let sub = Sub::raw_connect(&url, config.sub_connect_config)?;
        let mut out = Self::priv_new(remote_url.clone(), config.disconnect_on_slow_pong_ms);
        match out.priv_proc_wss_cli_result(tungstenite::client(
            tungstenite::handshake::client::Request {
                url: remote_url.into(),
                extra_headers: None,
            },
            sub.into_std_stream(),
        )) {
            Ok(_) => Ok(out),
            Err(e) if e.would_block() => Ok(out),
            Err(e) => Err(e),
        }
    }

    fn check_ready(&mut self) -> Result<bool> {
        self.priv_process()?;
        match self.state {
            Some(WssState::Ready(_)) => Ok(true),
            _ => Ok(false),
        }
    }

    fn remote_url(&self) -> Url2 {
        self.remote_url.clone()
    }

    fn read(&mut self, data: &mut WsFrame) -> Result<usize> {
        self.priv_process()?;
        log::trace!(
            "read from {} with connection state: {:?}",
            self.remote_url,
            self.state,
        );
        match &mut self.state {
            None => Err(ErrorKind::NotConnected.into()),
            Some(state) => match state {
                WssState::Ready(ref mut wss) => {
                    let r = wss.read_message();
                    log::trace!("read result from {}: {:?}", self.remote_url, r,);
                    match r {
                        Ok(msg) => {
                            data.assume(msg);

                            // our custom message handling breaks tungstenite's
                            // automatic pong responding - we need to inject
                            // it manually - skip our send queue
                            if let WsFrame::Ping(_) = &data {
                                priv_write_wss_frame(wss, WsFrame::Pong(Vec::with_capacity(0)))?;
                            }

                            // this is called last_pong - but really if
                            // we're receiving any messages, we know the
                            // connection is still open
                            self.last_pong = std::time::Instant::now();
                            Ok(1)
                        }
                        Err(tungstenite::error::Error::Io(e)) => {
                            if e.would_block() {
                                if let Some(pong_ms) = self.disconnect_on_slow_pong_ms {
                                    if self.last_pong.elapsed().as_millis() as u64 > pong_ms {
                                        return Err(Error::new(
                                            ErrorKind::TimedOut,
                                            format!("no messages received in {} ms", pong_ms),
                                        ));
                                    }
                                }
                            }
                            Err(e)
                        }
                        Err(e) => Err(Error::new(
                            ErrorKind::Other,
                            format!("tungstenite error: {:?}", e),
                        )),
                    }
                }
                _ => Err(Error::with_would_block()),
            },
        }
    }

    fn write(&mut self, data: WsFrame) -> Result<usize> {
        self.priv_process()?;
        self.write_buf.push_back(data);
        self.priv_write_pending()?;
        Ok(1)
    }

    fn flush(&mut self) -> Result<()> {
        loop {
            self.priv_process()?;
            self.priv_write_pending()?;
            if let Some(WssState::Ready(wss)) = &mut self.state {
                match wss.write_pending() {
                    Ok(_) => {
                        if self.write_buf.is_empty() {
                            return Ok(());
                        }
                    }
                    // data still in queue
                    Err(tungstenite::error::Error::Io(e)) if e.would_block() => (),
                    Err(tungstenite::error::Error::Io(e)) => return Err(e),
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("tungstenite error: {:?}", e),
                        ))
                    }
                }
            }
            std::thread::yield_now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_ginormsg(size: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(size);
        for i in 0..size {
            out.push((i % 256) as u8);
        }
        out
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

    fn suite<SubL: 'static + InStreamListenerStd, C: InStreamConfig>(
        mut listener: InStreamListenerWss<SubL>,
        c: C,
    ) {
        let (send_binding, recv_binding) = crossbeam_channel::unbounded();

        let server_thread = std::thread::spawn(move || {
            println!("bound to: {}", listener.binding());
            send_binding.send(listener.binding()).unwrap();

            let mut srv = loop {
                match listener.accept() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => std::thread::yield_now(),
                    Err(e) => panic!("{:?}", e),
                }
            };

            let rurl = srv.remote_url();
            assert_ne!(listener.binding(), rurl);
            assert_eq!(SCHEME, rurl.scheme());

            srv.write("hello from server".into()).unwrap();
            srv.flush().unwrap();

            let res = wait_read(&mut srv);
            assert_eq!("hello from client", res.as_str());

            srv.write(get_ginormsg(20000).into()).unwrap();
            srv.flush().unwrap();
        });

        let client_thread = std::thread::spawn(move || {
            let binding = recv_binding.recv().unwrap();
            println!("connect to: {}", binding);

            let mut cli: InStreamWss<SubL::StreamStd> =
                InStreamWss::connect(&binding, WssConnectConfig::new(c)).unwrap();

            assert_eq!(binding.as_str(), cli.remote_url().as_str());

            cli.write("hello from client".into()).unwrap();
            cli.flush().unwrap();

            let res = wait_read(&mut cli);
            assert_eq!("hello from server", res.as_str());

            let res = wait_read(&mut cli).as_bytes().to_vec();
            let ginormsg = get_ginormsg(20000);
            if ginormsg != res {
                let mut i = 0;
                loop {
                    if i >= res.len() || i >= ginormsg.len() {
                        break;
                    }
                    if res.get(i) != ginormsg.get(i) {
                        println!(
                            "mismatch at byte {}: {:?} != {:?}",
                            i,
                            res.get(i),
                            ginormsg.get(i),
                        );
                    }
                    i += 1;
                }
                panic!("expected {} bytes, got {} bytes", ginormsg.len(), res.len());
            }
        });

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        println!("done");
    }

    #[test]
    fn wss_works_mem() {
        let mut url = in_stream_mem::random_url("test");
        url.set_scheme(SCHEME).unwrap();
        let config = MemBindConfig::default();
        let config = TlsBindConfig::new(config).fake_certificate();
        let config = WssBindConfig::new(config);
        let l: InStreamListenerWss<InStreamListenerTls<InStreamListenerMem>> =
            InStreamListenerWss::bind(&url, config).unwrap();
        suite(l, TlsConnectConfig::new(MemConnectConfig::default()));
    }

    #[test]
    fn wss_works_tcp() {
        let config = TcpBindConfig::default();
        let config = TlsBindConfig::new(config).fake_certificate();
        let config = WssBindConfig::new(config);
        let l: InStreamListenerWss<InStreamListenerTls<InStreamListenerTcp>> =
            InStreamListenerWss::bind(&url2!("{}://127.0.0.1:0", SCHEME), config).unwrap();
        suite(l, TlsConnectConfig::new(TcpConnectConfig::default()));
    }

    #[test]
    fn wss_timout_on_slow_receive() {
        let mut url = in_stream_mem::random_url("test");
        url.set_scheme(SCHEME).unwrap();
        let config = MemBindConfig::default();
        let config = TlsBindConfig::new(config).fake_certificate();
        let config = WssBindConfig::new(config).disconnect_on_slow_pong_ms(Some(10));
        let mut listener: InStreamListenerWss<InStreamListenerTls<InStreamListenerMem>> =
            InStreamListenerWss::bind(&url, config).unwrap();
        let binding = listener.binding();
        println!("got binding: {}", binding);
        let server_thread = std::thread::spawn(move || {
            let mut srv = loop {
                match listener.accept() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => std::thread::yield_now(),
                    Err(e) => panic!("{:?}", e),
                }
            };

            let mut frame = WsFrame::default();
            loop {
                match srv.read(&mut frame) {
                    Ok(_) => (),
                    Err(e) if e.would_block() => {
                        std::thread::yield_now();
                    }
                    Err(e) => {
                        assert_eq!(
                            "Custom { kind: TimedOut, error: \"no messages received in 10 ms\" }",
                            &format!("{:?}", e),
                        );
                        break;
                    }
                }
            }
        });

        let mut cli: InStreamWss<InStreamTls<InStreamMem>> = InStreamWss::connect(
            &binding,
            WssConnectConfig::new(TlsConnectConfig::new(MemConnectConfig::default())),
        )
        .unwrap();

        cli.write("hello from client".into()).unwrap();
        cli.flush().unwrap();

        server_thread.join().unwrap();
    }

    #[test]
    fn no_wss_timout_on_active_receive() {
        let mut url = in_stream_mem::random_url("test");
        url.set_scheme(SCHEME).unwrap();
        let config = MemBindConfig::default();
        let config = TlsBindConfig::new(config).fake_certificate();
        let config = WssBindConfig::new(config).disconnect_on_slow_pong_ms(Some(10));
        let mut listener: InStreamListenerWss<InStreamListenerTls<InStreamListenerMem>> =
            InStreamListenerWss::bind(&url, config).unwrap();
        let binding = listener.binding();
        println!("got binding: {}", binding);
        let server_thread = std::thread::spawn(move || {
            let mut srv = loop {
                match listener.accept() {
                    Ok(srv) => break srv,
                    Err(e) if e.would_block() => std::thread::yield_now(),
                    Err(e) => panic!("{:?}", e),
                }
            };

            let mut frame = WsFrame::default();
            loop {
                match srv.read(&mut frame) {
                    Ok(_) => (),
                    Err(e) if e.would_block() => {
                        std::thread::yield_now();
                    }
                    Err(e) => {
                        assert_eq!(
                            "Custom { kind: Other, error: \"tungstenite error: Protocol(\\\"Connection reset without closing handshake\\\")\" }",
                            &format!("{:?}", e),
                        );
                        break;
                    }
                }
            }
        });

        let mut cli: InStreamWss<InStreamTls<InStreamMem>> = InStreamWss::connect(
            &binding,
            WssConnectConfig::new(TlsConnectConfig::new(MemConnectConfig::default())),
        )
        .unwrap();

        cli.write("hello from client".into()).unwrap();
        cli.flush().unwrap();

        let client_thread = std::thread::spawn(move || {
            let mut frame = WsFrame::default();
            let start = std::time::Instant::now();
            loop {
                match cli.read(&mut frame) {
                    Ok(_) => (),
                    Err(e) if e.would_block() => {
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                    Err(e) => panic!("{:?}", e),
                }
                if start.elapsed().as_millis() > 30 {
                    break;
                }
            }
        });

        server_thread.join().unwrap();
        client_thread.join().unwrap();
    }
}
