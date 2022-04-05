use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //Bubble up the error if we failed to bind the address
    // otherwise call await on our server
    run()?.await
}
