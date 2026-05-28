use std::io::Result;

use log::{info, trace};
use tokio::{
    join, runtime,
    sync::mpsc::{self, UnboundedReceiver},
};

use crate::transport::{InputTransport, lan::LanInputTransport, models::InputMessage};

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
        info!("Server started successfully");
        join!(
            lan_transport.run(input_sender),
            Self::read_input_signals(&mut input_receiver)
        );

        Ok(())
    }

    async fn read_input_signals(input_receiver: &mut UnboundedReceiver<InputMessage>) {
        while let Some(msg) = input_receiver.recv().await {
            trace!("Signal received: {:?}", msg.signal);
        }
    }
}
