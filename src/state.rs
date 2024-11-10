//! Server state.

/// An enum representing the current server state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerState {
    /// Handshake state.
    ///
    /// Initial state of the server, will determine the next state:
    /// [`ServerState::Status`] or [`ServerState::Login`].
    Handshake,

    /// Status state.
    ///
    /// After [`ServerState::Handshake`], used for the server list ping.
    Status,

    /// Login state.
    ///
    /// After [`ServerState::Handshake`]. Authentication of the player before
    /// [`ServerState::Play`] state.
    Login,

    /// Play state.
    ///
    /// Active state of the server, after login [`ServerState::Login`].
    Play,

    /// Connection closed.
    Closed,
}
