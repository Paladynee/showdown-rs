use crate::components;

pub fn parse_websocket_message(message: &str) -> Option<components::WebSocketClientData> {
    let parsed = serde_json::from_str(message);
    // if let Ok(msg) = &parsed {
    //     eprintln!("Recieved message: {:#?}", msg);
    // }
    parsed.ok()
}
