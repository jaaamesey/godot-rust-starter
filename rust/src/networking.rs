use std::{pin::Pin, task::Context, time::Duration};

use futures::{FutureExt, TryFutureExt, select};
use futures_timer::Delay;
use godot::{
    classes::Timer,
    obj::NewAlloc,
    prelude::*,
    task::{SignalFuture, TaskHandle},
};
use matchbox_socket::{PeerState, WebRtcSocket};

const CHANNEL_ID: usize = 0;

#[derive(GodotClass)]
#[class(base=Node)]
struct Networking {
    socket: Option<WebRtcSocket>,
    loop_fut: Option<
        Pin<Box<dyn futures::Future<Output = std::result::Result<(), matchbox_socket::Error>>>>,
    >,
}

#[godot_api]
impl INode for Networking {
    fn init(_: Base<Node>) -> Self {
        Self {
            socket: None,
            loop_fut: None,
        }
    }
    fn process(&mut self, _delta: f64) {
        if let Some(future) = &mut self.loop_fut {
            // This seems to be what fails

            let _ = future.try_poll_unpin(&mut Context::from_waker(&futures::task::noop_waker()));
        }
        if let Some(socket) = &mut self.socket {
            // Handle any new peers
            for (peer, state) in socket.try_update_peers().unwrap_or_default() {
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
        // #[cfg(target_arch = "wasm32")]
        // console_error_panic_hook::set_once();
        // #[cfg(target_arch = "wasm32")]
        // console_log::init_with_level(log::Level::Debug).unwrap();

        let (mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://127.0.0.1:3536/");

        self.socket = Some(socket);

        self.loop_fut = Some(loop_fut);

        // drop(loop_fut); // We don't want to await this

        // #[cfg(not(target_arch = "wasm32"))]
        // godot::task::spawn(async_main());
        // #[cfg(target_arch = "wasm32")]
        // godot::task::spawn(async_main());
    }
}
//
// async fn async_main() {
//     godot_print!("Connecting to matchbox");
//     let (mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://127.0.0.1:3536/");
//
//     let loop_fut = loop_fut.fuse();
//     futures::pin_mut!(loop_fut);
//
//     // This is what WASM has trouble with. Can we use godot's timers instead somehow?
//     let timeout = Delay::new(Duration::from_millis(16));
//     futures::pin_mut!(timeout);
//
//     let mut timer = Timer::new_alloc();
//     // // Configure the timer:
//     // // Set the wait time in seconds (16ms = 0.016 seconds)
//     // timer.set_wait_time(0.016);
//     // timer.set_one_shot(true);
//     // let timeout = timer.signals().timeout().to_future();
//     // timer.start();
//     loop {
//         // Handle any new peers
//         for (peer, state) in socket.update_peers() {
//             match state {
//                 PeerState::Connected => {
//                     godot_print!("Peer joined: {peer}");
//                     let packet = "hello friend!".as_bytes().to_vec().into_boxed_slice();
//                     socket.channel_mut(CHANNEL_ID).send(packet, peer);
//                 }
//                 PeerState::Disconnected => {
//                     godot_print!("Peer left: {peer}");
//                 }
//             }
//         }
//
//         // Accept any messages incoming
//         for (peer, packet) in socket.channel_mut(CHANNEL_ID).receive() {
//             let message = String::from_utf8_lossy(&packet);
//             godot_print!("Message from {peer}: {message:?}");
//         }
//         select! {
//             // Restart this loop every 100ms
//             _ = (&mut timeout).fuse() => {
//                 timeout.reset(Duration::from_millis(100));
//             }
//
//             // Or break if the message loop ends (disconnected, closed, etc.)
//             _ = &mut loop_fut => {
//                 break;
//             }
//         }
//     }
// }
//
// use std::time::Duration;
