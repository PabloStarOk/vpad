use std::{io::Result, net::SocketAddr};

use log::{error, info};
use tokio::{
    runtime, select,
    sync::mpsc::{self, UnboundedReceiver},
};

use crate::{
    transport::{
        InputTransport,
        lan::LanInputTransport,
        models::{ClientPacket, ServerPacket},
    },
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

        let (input_sender, mut input_receiver) = mpsc::unbounded_channel::<ClientPacket>();
        let (lan_output_sender, mut lan_output_receiver) =
            mpsc::unbounded_channel::<ServerPacket<SocketAddr>>();

        let lan_transport = LanInputTransport::new(tk_handle).await;
        let mut gamepad_manager = GamepadManager::new(lan_output_sender);

        info!("Server started successfully");
        select! {
            _ = lan_transport.run(input_sender, &mut lan_output_receiver) => {
                error!("LAN input transport stopped unexpectedly");
            },
            _ = gamepad_manager.run(&mut input_receiver) => {
                error!("Gamepad manager stopped unexpectedly");
            },
            _ = Self::wait_shutdown_signal() => {},
        }

        self.shutdown(
            &mut gamepad_manager,
            &lan_transport,
            &mut lan_output_receiver,
        )
        .await;

        Ok(())
    }

    async fn wait_shutdown_signal() {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut sigterm =
                signal(SignalKind::terminate()).expect("Could not subscribe to terminate signals");
            let mut sigint =
                signal(SignalKind::interrupt()).expect("Could not subscribe to interrupt signals");
            select! {
                _ = sigterm.recv() => {},
                _ = sigint.recv() => {}
            }
        }
        #[cfg(windows)]
        {
            use tokio::signal;
            signal::ctrl_c()
                .await
                .expect("Could not subscribe to CTRL + C signals")
        }
    }

    async fn shutdown(
        &self,
        gamepad_manager: &mut GamepadManager,
        lan_transport: &LanInputTransport,
        lan_output_receiver: &mut UnboundedReceiver<ServerPacket<SocketAddr>>,
    ) {
        info!("Shutting down server");
        gamepad_manager.shutdown();
        lan_transport.shutdown(lan_output_receiver).await;
    }
}
