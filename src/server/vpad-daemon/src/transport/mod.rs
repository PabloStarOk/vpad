use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::transport::models::{ServerPacket, VpadPacket};

pub mod lan;
pub mod models;

pub trait InputTransport<T>: Send {
    fn run(
        &self,
        input_sender: UnboundedSender<VpadPacket>,
        output_receiver: &mut UnboundedReceiver<ServerPacket<T>>,
    ) -> impl Future<Output = ()>;

    fn shutdown(
        &self,
        output_receiver: &mut UnboundedReceiver<ServerPacket<T>>,
    ) -> impl Future<Output = ()>;
}
