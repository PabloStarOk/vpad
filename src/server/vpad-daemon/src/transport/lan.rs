use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use libmdns::Responder;

use log::{debug, error, info, trace, warn};
use tokio::{
    join,
    net::UdpSocket,
    runtime::Handle,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time,
};

use crate::transport::{
    InputTransport,
    models::{ClientId, ClientMessage, ClientPacket, ServerMessage, ServerPacket},
};

pub struct LanInputTransport {
    tk_handle: Handle,
    udp_socket: UdpSocket,
}

impl LanInputTransport {
    const HOST_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    const HOST_PORT: u16 = 0;
    const SOCKET_ADDR: SocketAddrV4 = SocketAddrV4::new(Self::HOST_ADDR, Self::HOST_PORT);
    const SERVICE_DOMAIN: &str = "_vpad._udp";
    const SERVICE_NAME: &str = "VpadServer";
    const SHUTDOWN_TIMEOUT_MS: Duration = Duration::from_millis(5000);

    pub async fn new(tk_handle: Handle) -> Self {
        let udp_socket = UdpSocket::bind(Self::SOCKET_ADDR)
            .await
            .expect("Could not bind UDP socket to the specified socket address.");
        LanInputTransport {
            tk_handle,
            udp_socket,
        }
    }

    async fn listen(&self, packet_sender: UnboundedSender<ClientPacket>) {
        let mut data_buf = [0u8; ClientMessage::MAX_SIZE];
        loop {
            let addr = match self.udp_socket.recv_from(&mut data_buf).await {
                Ok((_, address)) => address,
                Err(error) => {
                    error!("Could not read data of UDP packet: {error}");
                    continue;
                }
            };

            trace!("Received UDP packet from {}.", addr.ip());

            let message = match ClientMessage::try_from(&data_buf) {
                Ok(msg) => msg,
                Err(error) => {
                    error!("Could not create message from data of UDP packet: {error}");
                    continue;
                }
            };

            let packet = ClientPacket {
                client_id: ClientId::Network(addr),
                message,
            };
            packet_sender
                .send(packet)
                .expect("Could not send message on the unbounded channel.");
        }
    }

    async fn send(&self, output_receiver: &mut UnboundedReceiver<ServerPacket<SocketAddr>>) {
        while let Some(packet) = output_receiver.recv().await {
            let buffer = match packet.message {
                ServerMessage::Shutdown => [packet.message as u8],
            };

            if let Err(error) = self.udp_socket.send_to(&buffer, packet.client_id).await {
                error!(
                    "Could not send message '{:?}' to client {} | Error: {}",
                    packet.message, packet.client_id, error
                );
                continue;
            };

            debug!(
                "Message '{:?}' sent to client {}",
                packet.message, packet.client_id,
            )
        }
    }
}

impl InputTransport<SocketAddr> for LanInputTransport {
    async fn run(
        &self,
        packet_sender: UnboundedSender<ClientPacket>,
        output_receiver: &mut UnboundedReceiver<ServerPacket<SocketAddr>>,
    ) {
        let socket_port = self
            .udp_socket
            .local_addr()
            .expect("Could not get the local address of the UDP socket")
            .port();
        info!("Running UDP listener on port {socket_port}");

        let responder = Responder::spawn(&self.tk_handle)
            .expect("Could not create responder to register service to mDNS.");
        let _dmns_service =
            responder.register(Self::SERVICE_DOMAIN, Self::SERVICE_NAME, socket_port, &[]);

        join!(self.listen(packet_sender), self.send(output_receiver));
    }

    /// Shutdowns LAN input transport flushing all buffered Shutdown messages.
    /// The channel must be closed for this to work properly.
    async fn shutdown(&self, output_receiver: &mut UnboundedReceiver<ServerPacket<SocketAddr>>) {
        let timeout = time::timeout(Self::SHUTDOWN_TIMEOUT_MS, self.send(output_receiver));
        if timeout.await.is_err() {
            warn!("Could not send buffered server messages to clients due to a timeout");
        }

        debug!("LAN Input transport stopped")
    }
}
