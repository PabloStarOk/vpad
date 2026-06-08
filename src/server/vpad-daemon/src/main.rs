use std::io::Result;

use vpad_daemon::server::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut server = Server::new().await;
    server.run().await
}
