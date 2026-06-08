use tokio::sync::mpsc::UnboundedReceiver;

use crate::transport::models::ServerPacket;

pub mod lan;
pub mod models;

pub trait Transport<T>: Send {
    fn run(
        &self,
        output_receiver: &mut UnboundedReceiver<ServerPacket<T>>,
    ) -> impl Future<Output = ()>;

    fn shutdown(
        &self,
        output_receiver: &mut UnboundedReceiver<ServerPacket<T>>,
    ) -> impl Future<Output = ()>;
}
