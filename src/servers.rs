use socketioxide::extract::SocketRef;
use std::sync::{Arc, RwLock};
use crate::GameServer;

/// Initializes handlers for a connected game server
pub fn init(socket: SocketRef, servers: Arc<RwLock<Vec<GameServer>>>) {
    // Set up disconnect handler
    socket.on("disconnect", move |_| {
        let mut servers = servers.write().unwrap();
        if let Some(pos) = servers.iter().position(|s| s.socket.id == socket.id) {
            servers.remove(pos);
        }
    });
}