use std::net::TcpListener;
use zero2prod::config;
use zero2prod::startup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //Bubble up the error if we failed to bind the address
    // otherwise call await on our server
    let configuration = config::get_configuration().expect("Could not read configuration file");
    let addr = format!("127.0.0.1:{}", configuration.app_port);
    let listener = TcpListener::bind(addr)?;
    startup::run(listener)?.await
}
