use lib::ws::*;

use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    start_ws_server().await
}
