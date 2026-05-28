use std::io::Result;

use vpad_daemon::server::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let server = Server::new();
    server.run().await
}
