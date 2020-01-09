use crate::*;
use backtrace::Backtrace;

/// listen / accept new connections from a server socket
/// timing strategy:
///   - while there are new connections, keep going for 20 ms, then yield
///   - if WouldBlock, sleep for 5 ms
pub(crate) async fn listen_job(
    mut listen: TcpWssServer,
    wss_send: crossbeam_channel::Sender<TcpWss>,
) {
    let mut last_break = std::time::Instant::now();
    loop {
        match listen.accept() {
            Ok(wss) => {
                if !wss_send.i_send(wss) {
                    return;
                }
            }
            Err(e) if e.would_block() => {
                last_break = std::time::Instant::now();
                futures_timer::Delay::new(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => {
                error!(
                    "LISTEN ACCEPT FAIL: {:?}\nbacktrace: {:?}",
                    e,
                    Backtrace::new()
                );
                // don't panic : )
                // we just want to drop this connection, so do nothing
            }
        }
        if last_break.elapsed().as_millis() > 20 {
            last_break = std::time::Instant::now();
            TaskYield::new().await;
        }
    }
}
