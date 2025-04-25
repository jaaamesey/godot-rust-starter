use std::time::Duration;

use futures::{FutureExt, select};
use futures_timer::Delay;
use godot::prelude::*;
use matchbox_socket::{PeerState, WebRtcSocket};

const CHANNEL_ID: usize = 0;

#[derive(GodotClass)]
#[class(base=Node)]
struct Networking {
    socket: Option<WebRtcSocket>,
}

#[godot_api]
impl INode for Networking {
    fn init(_: Base<Node>) -> Self {
        Self { socket: None }
    }

    fn physics_process(&mut self, _delta: f64) {
        if let Some(socket) = self.socket.as_mut() {
            // Accept any messages incoming
            for (peer, packet) in socket.channel_mut(CHANNEL_ID).receive() {
                let message = String::from_utf8_lossy(&packet);
                godot_print!("Message from {peer}: {message:?}");
            }

            // Handle any new peers
            let peers = socket.try_update_peers();
            if let Err(err) = peers {
                godot_print!("Socket is closed");
                return;
            }
            for (peer, state) in peers.unwrap() {
                match state {
                    PeerState::Connected => {
                        godot_print!("Peer joined: {peer}");
                        let packet = "hello friend!".as_bytes().to_vec().into_boxed_slice();
                        socket.channel_mut(CHANNEL_ID).send(packet, peer);
                    }
                    PeerState::Disconnected => {
                        godot_print!("Peer left: {peer}");
                    }
                }
            }

            // Accept any messages incoming
            for (peer, packet) in socket.channel_mut(CHANNEL_ID).receive() {
                let message = String::from_utf8_lossy(&packet);
                godot_print!("Message from {peer}: {message:?}");
            }
        }
    }
}

#[godot_api]
impl Networking {
    #[func]
    fn start(&mut self) {
        godot_print!("Starting connection...");

        // Works!!
        let (socket, loop_fut) =
            matchbox_socket::WebRtcSocket::new_unreliable("ws://127.0.0.1:3536");

        self.socket = Some(socket);

        godot::task::spawn(async move {
            let res = loop_fut.await;
            match res {
                Ok(_) => {
                    godot_print!("Successfully awaited");
                }
                Err(e) => {
                    godot_print!("Error awaiting");
                }
            }
        });
    }
}
