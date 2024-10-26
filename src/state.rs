#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerState {
    Handshake,
    Status,
    Login,
    Play,
    Closed,
}
