use std::io::Result;

use log::info;
use tokio::{
    join, runtime,
    sync::mpsc::{self},
};

use crate::{
    transport::{InputTransport, lan::LanInputTransport, models::InputMessage},
    virt::gamepad::GamepadManager,
};

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Server {}
    }

    pub async fn run(&self) -> Result<()> {
        env_logger::init();
        let tk_handle = runtime::Handle::current();
        let (input_sender, mut input_receiver) = mpsc::unbounded_channel::<InputMessage>();
        let lan_transport = LanInputTransport::new(tk_handle);
        let mut gamepad_manager = GamepadManager::new();
        info!("Server started successfully");
        join!(
            lan_transport.run(input_sender),
            gamepad_manager.manage(&mut input_receiver)
        );

        Ok(())
    }
}
