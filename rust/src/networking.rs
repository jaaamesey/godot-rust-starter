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
                godot_print!("HANDLING PEER");
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
            //godot_print!("{}", socket.connected_peers().count());

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

        // // Works!!
        // let (socket, loop_fut) =
        //     matchbox_socket::WebRtcSocket::new_unreliable("ws://localhost:3536/some_lobby?next=2");

        godot::task::spawn(async_main());
    }
}

async fn async_main() {
    godot_print!("Connecting to matchbox");
    let (mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://localhost:3536/");

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(16));
    futures::pin_mut!(timeout);

    loop {
        // Handle any new peers
        for (peer, state) in socket.update_peers() {
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

        select! {
            // Restart this loop every 100ms
            _ = (&mut timeout).fuse() => {
                timeout.reset(Duration::from_millis(100));
            }

            // Or break if the message loop ends (disconnected, closed, etc.)
            _ = &mut loop_fut => {
                break;
            }
        }
    }
}
