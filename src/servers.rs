use socketioxide::extract::SocketRef;
use std::sync::{Arc, RwLock};
use crate::ChildServer;

pub fn init(socket: SocketRef, servers: Arc<RwLock<Vec<ChildServer>>>) {
    socket.on_disconnect(move |s| {
        on_disconnect(s, servers.clone())
    });}

pub fn on_disconnect(socket: SocketRef, servers: Arc<RwLock<Vec<ChildServer>>>) {
   let mut servers = servers.write().unwrap();
   if let Some(pos) = servers.iter().position(|s| s.socket.as_ref().unwrap().id == socket.id) {
       servers.remove(pos);
   }
}