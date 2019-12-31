use std::time::Instant;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalkmanLogItem {
    #[serde(with = "serde_millis")]
    time: Instant,
    event: WalkmanEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WalkmanEvent {
    // CoreEvent(Action),
    Sim2hEvent(WalkmanSim2hEvent),
}


pub fn walkman_log_sim2h(url: Url, data: WalkmanSim2hEvent) -> WalkmanLogItem {
    WalkmanLogItem {
        time: Instant::now(),
        event: WalkmanEvent::Sim2hEvent(data),
    }
}

// trait WalkmanLogger<T> {
//     fn log(data: T) -> WalkmanLogItem;
// }

// pub struct WalkmanSim2hLogger {
//     url: Url,
// }

// impl WalkmanLogger<WalkmanSim2hEvent> for WalkmanSim2hLogger {
//     fn log(data: WalkmanSim2hEvent) -> WalkmanLogItem {
//         WalkmanLogItem {
//             time: Instant::now(),
//             event: WalkmanEvent::Sim2hEvent(data),
//         }
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WalkmanSim2hEvent {
    Connect(Url),
    Disconnect(Url),
    Message(Url, WireMessage),
}
