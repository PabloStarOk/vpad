use tokio::sync::mpsc::UnboundedSender;

use crate::transport::models::InputMessage;

pub mod lan;
pub mod models;

pub trait InputTransport: Send {
    fn run(&self, input_sender: UnboundedSender<InputMessage>) -> impl Future<Output = ()>;
}
