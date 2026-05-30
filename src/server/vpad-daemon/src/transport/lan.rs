use std::net::{Ipv4Addr, SocketAddrV4};

use libmdns::Responder;

use log::{error, info, trace};
use tokio::{net::UdpSocket, runtime::Handle, sync::mpsc::UnboundedSender};

use crate::transport::{
    InputTransport,
    models::{ClientId, Message, VpadPacket},
};

pub struct LanInputTransport {
    tk_handle: Handle,
}

impl LanInputTransport {
    const HOST_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    const HOST_PORT: u16 = 0;
    const SOCKET_ADDR: SocketAddrV4 = SocketAddrV4::new(Self::HOST_ADDR, Self::HOST_PORT);
    const SERVICE_DOMAIN: &str = "_vpad._udp";
    const SERVICE_NAME: &str = "VpadServer";

    pub fn new(tk_handle: Handle) -> Self {
        LanInputTransport { tk_handle }
    }

    async fn listen(socket: UdpSocket, packet_sender: UnboundedSender<VpadPacket>) {
        let mut data_buf = [0u8; Message::MAX_SIZE];
        loop {
            let addr = match socket.recv_from(&mut data_buf).await {
                Ok((_, address)) => address,
                Err(error) => {
                    error!("Could not read data of UDP packet: {error}");
                    continue;
                }
            };

            trace!("Received UDP packet from {}.", addr.ip());

            let message = match Message::try_from(&data_buf) {
                Ok(msg) => msg,
                Err(error) => {
                    error!("Could not create message from data of UDP packet: {error}");
                    continue;
                }
            };

            let packet = VpadPacket {
                client_id: ClientId::Network(addr.ip()),
                message,
            };
            packet_sender
                .send(packet)
                .expect("Could not send message on the unbounded channel.");
        }
    }
}

impl InputTransport for LanInputTransport {
    async fn run(&self, packet_sender: UnboundedSender<VpadPacket>) {
        let udp_socket = UdpSocket::bind(Self::SOCKET_ADDR)
            .await
            .expect("Could not bind UDP socket to the specified socket address.");
        let socket_port = udp_socket
            .local_addr()
            .expect("Could not get the local address of the UDP socket")
            .port();
        info!("Running UDP listener on port {socket_port}");

        let responder = Responder::spawn(&self.tk_handle)
            .expect("Could not create responder to register service to mDNS.");
        let _dmns_service =
            responder.register(Self::SERVICE_DOMAIN, Self::SERVICE_NAME, socket_port, &[]);

        Self::listen(udp_socket, packet_sender).await;
    }
}
