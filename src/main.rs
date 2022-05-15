use std::io::stdout;
use zero2prod::config;
use zero2prod::startup::Application;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), stdout);
    telemetry::init_subscriber(subscriber);
    //Bubble up the error if we failed to bind the address
    // otherwise call await on our server
    let configuration = config::get_configuration().expect("Could not read configuration file");
    let application = Application::build(configuration)
        .await?;
    application.run_until_stopped().await?;
    Ok(())
}
