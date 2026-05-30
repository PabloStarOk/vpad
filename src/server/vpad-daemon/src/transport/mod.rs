use tokio::sync::mpsc::UnboundedSender;

use crate::transport::models::VpadPacket;

pub mod lan;
pub mod models;

pub trait InputTransport: Send {
    fn run(&self, input_sender: UnboundedSender<VpadPacket>) -> impl Future<Output = ()>;
}
