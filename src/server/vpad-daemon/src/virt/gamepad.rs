use std::collections::HashMap;

use crate::{
    transport::models::{ClientId, ConnectionMessage, InputSignal, Message, VpadPacket},
    virt::linux::LinuxVirtualGamepad,
};

use log::{debug, error, info, trace};
use tokio::sync::mpsc::UnboundedReceiver;

#[cfg(target_os = "linux")]
type VirtualGamepad = LinuxVirtualGamepad;

pub struct GamepadManager {
    gamepads: HashMap<ClientId, VirtualGamepad>,
}

impl GamepadManager {
    pub fn new() -> Self {
        GamepadManager {
            gamepads: HashMap::new(),
        }
    }

    pub async fn manage(&mut self, packet_receiver: &mut UnboundedReceiver<VpadPacket>) {
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
