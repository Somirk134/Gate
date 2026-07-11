use gate_server::logging;

#[tokio::main]
async fn main() {
    logging::init();
    logging::startup_line("process starting");

    if std::env::args().any(|arg| arg == "--healthcheck") {
        match gate_server::healthcheck().await {
            Ok(()) => std::process::exit(0),
            Err(error) => logging::fatal(format!("healthcheck failed: {error:#}")),
        }
    }

    if let Err(error) = gate_server::ServerBootstrap::new().boot().await {
        logging::fatal(format!("{error:#}"));
    }
}
