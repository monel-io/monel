use crate::config::Config;
use crate::errors::ServerError;

/// Starts the HTTP server.
pub fn serve(config: &Config) -> Result<(), ServerError> {
    // Check port availability
    log::info!("starting server on {}:{}", config.host, config.port);

    // In a real implementation, this would bind the socket, configure TLS, etc.
    // The AI agent has to read this file to understand what side effects `serve` has,
    // even though it may only care about `authenticate`.

    if config.tls.is_some() {
        log::info!("TLS enabled");
    }

    log::info!("server listening on port {}", config.port);
    Ok(())
}
