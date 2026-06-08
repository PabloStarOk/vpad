use std::{collections::HashMap, net::SocketAddr};

use crate::{
    transport::models::{
        ClientId, ConnectionMessage, InputSignal, Message, ServerMessage, ServerPacket, VpadPacket,
    },
    virt::linux::LinuxVirtualGamepad,
};

use log::{debug, error, info, trace};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[cfg(target_os = "linux")]
type VirtualGamepad = LinuxVirtualGamepad;

pub struct GamepadManager {
    gamepads: HashMap<ClientId, VirtualGamepad>,
    opt_lan_msg_sender: Option<UnboundedSender<ServerPacket<SocketAddr>>>,
}

impl GamepadManager {
    pub fn new(lan_msg_sender: UnboundedSender<ServerPacket<SocketAddr>>) -> Self {
        GamepadManager {
            gamepads: HashMap::new(),
            opt_lan_msg_sender: Option::Some(lan_msg_sender),
        }
    }

    pub async fn run(&mut self, packet_receiver: &mut UnboundedReceiver<VpadPacket>) {
        while let Some(packet) = packet_receiver.recv().await {
            trace!("Received VPad packet: {:?}", packet);
            match packet.message {
                Message::Connection(msg_type) => {
                    self.handle_connection_msg(packet.client_id, msg_type)
                }
                Message::Input(signal) => match self.gamepads.get_mut(&packet.client_id) {
                    Some(gamepad) => Self::try_report_input(gamepad, packet.client_id, signal),
                    None => trace!(
                        "Ignoring input received from unknown client {:?}",
                        packet.client_id
                    ),
                },
            }
        }
    }

    pub fn shutdown(&mut self) {
        let Some(lan_msg_sender) = &self.opt_lan_msg_sender else {
            return;
        };

        self.gamepads.drain().for_each(|(id, _)| {
            let result = match id {
                ClientId::Network(addr) => lan_msg_sender.send(ServerPacket {
                    client_id: addr,
                    message: ServerMessage::Shutdown,
                }),
            };

            if let Err(error) = result {
                error!(
                    "Could not send shutdown message to client '{:?}' | Error: {}",
                    id, error
                );
            };
        });

        self.opt_lan_msg_sender = None;
    }

    fn handle_connection_msg(&mut self, client_id: ClientId, msg: ConnectionMessage) {
        match msg {
            ConnectionMessage::Connect => {
                if self.gamepads.contains_key(&client_id) {
                    return;
                }

                let gamepad = match VirtualGamepad::new() {
                    Ok(gamepad) => gamepad,
                    Err(error) => {
                        error!("Could not create virtual device: {error}");
                        return;
                    }
                };
                self.gamepads.insert(client_id, gamepad);
                debug!("New virtual gamepad created for client {:?}", client_id);
            }
            ConnectionMessage::Disconnect => {
                if self.gamepads.remove(&client_id).is_some() {
                    info!("Device disconnected for client: {:?}", client_id)
                }
            }
        }
    }

    fn try_report_input(gamepad: &mut VirtualGamepad, client_id: ClientId, signal: InputSignal) {
        if let Err(error) = gamepad.report_input(signal) {
            error!(
                "Error when trying to report input signal {:?} for client {:?} | Error: {}",
                signal, client_id, error
            )
        }
    }
}
