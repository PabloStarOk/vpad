use std::collections::HashMap;

use crate::{
    transport::models::{ClientId, InputMessage},
    virt::linux::LinuxVirtualGamepad,
};

use log::{debug, error, trace};
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

    pub async fn manage(&mut self, input_receiver: &mut UnboundedReceiver<InputMessage>) {
        while let Some(msg) = input_receiver.recv().await {
            trace!("Received input message: {:?}", msg);
            match self.gamepads.get_mut(&msg.client_id) {
                Some(gamepad) => Self::try_report_input(gamepad, msg),
                None => {
                    let mut gamepad = match VirtualGamepad::new() {
                        Ok(gamepad) => gamepad,
                        Err(error) => {
                            error!("Could not create virtual. {error}");
                            return;
                        }
                    };
                    Self::try_report_input(&mut gamepad, msg);
                    self.gamepads.insert(msg.client_id, gamepad);
                    debug!("New virtual gamepad created for client {:?}", msg.client_id);
                }
            }
        }
    }

    fn try_report_input(gamepad: &mut VirtualGamepad, msg: InputMessage) {
        match gamepad.report_input(msg.signal) {
            Ok(()) => {}
            Err(error) => error!(
                "Error when trying to report input signal for message {:?} | Error: {}",
                msg, error
            ),
        }
    }
}
