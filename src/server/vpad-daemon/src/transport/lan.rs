use std::net::{Ipv4Addr, SocketAddrV4};

use libmdns::Responder;

use log::{error, info, trace};
use tokio::{net::UdpSocket, runtime::Handle, sync::mpsc::UnboundedSender};

use crate::transport::{
    InputTransport,
    models::{ClientId, InputMessage, InputSignal},
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

    async fn listen(socket: UdpSocket, input_sender: UnboundedSender<InputMessage>) {
        let mut data_buf = [0; InputSignal::SIZE];
        loop {
            let addr = match socket.recv_from(&mut data_buf).await {
                Ok((_, address)) => address,
                Err(error) => {
                    error!("Could not read data of UDP packet: {error}");
                    continue;
                }
            };

            trace!("Received UDP packet from {}.", addr.ip());

            let input_signal = match InputSignal::try_from(&data_buf) {
                Ok(signal) => signal,
                Err(error) => {
                    error!("Could not create input signal from data of UDP packet: {error}");
                    continue;
                }
            };

            let input_msg = InputMessage {
                client_id: ClientId::Network(addr.ip()),
                signal: input_signal,
            };
            input_sender
                .send(input_msg)
                .expect("Could not send input message on the unbounded channel.");
        }
    }
}

impl InputTransport for LanInputTransport {
    async fn run(&self, input_sender: UnboundedSender<InputMessage>) {
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

        Self::listen(udp_socket, input_sender).await;
    }
}
