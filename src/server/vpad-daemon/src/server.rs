use std::{io::Result, net::SocketAddr};

use log::{error, info};
use tokio::{
    runtime, select,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use crate::{
    transport::{
        Transport,
        lan::LanTransport,
        models::{ClientPacket, ServerPacket},
    },
    virt::gamepad::GamepadManager,
};

pub struct Server {
    lan_transport: LanTransport,
    gamepad_manager: GamepadManager,
    client_packet_tx: UnboundedSender<ClientPacket>,
    lan_server_packet_rx: UnboundedReceiver<ServerPacket<SocketAddr>>,
}

impl Server {
    pub async fn new() -> Self {
        let tk_handle = runtime::Handle::current();

        let (client_packet_tx, client_packet_rx) = mpsc::unbounded_channel::<ClientPacket>();
        let (lan_server_packet_tx, lan_server_packet_rx) =
            mpsc::unbounded_channel::<ServerPacket<SocketAddr>>();

        let lan_transport = LanTransport::new(tk_handle).await;
        let gamepad_manager = GamepadManager::new(client_packet_rx, lan_server_packet_tx);

        Server {
            lan_transport,
            gamepad_manager,
            client_packet_tx,
            lan_server_packet_rx,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        env_logger::init();

        info!("Server started successfully");
        select! {
            _ = self.lan_transport.run(self.client_packet_tx.clone(), &mut self.lan_server_packet_rx) => {
                error!("LAN transport stopped unexpectedly");
            },
            _ = self.gamepad_manager.run() => {
                error!("Gamepad manager stopped unexpectedly");
            },
            _ = Self::wait_shutdown_signal() => {},
        }

        self.shutdown().await;

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

    async fn shutdown(&mut self) {
        info!("Shutting down server");
        self.gamepad_manager.shutdown();
        self.lan_transport
            .shutdown(&mut self.lan_server_packet_rx)
            .await;
    }
}
