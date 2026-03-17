use std::time::Instant;

/// Routes and processes an incoming HTTP request.
///
/// Returns 404 for unmatched routes, 500 for internal errors.
/// Logs request duration for observability.
pub fn handle_request(method: &str, path: &str) -> Response {
    let start = Instant::now();

    let response = match route(method, path) {
        None => Response {
            status: 404,
            body: "not found".into(),
        },
        Some(handler) => match handler() {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("request_failed: path={}, error={}", path, e);
                Response {
                    status: 500,
                    body: "internal error".into(),
                }
            }
        },
    };

    let elapsed = start.elapsed();
    log::info!(
        "request_complete: method={}, path={}, status={}, duration_ms={}",
        method,
        path,
        response.status,
        elapsed.as_millis()
    );

    response
}

pub struct Response {
    pub status: u16,
    pub body: String,
}

type Handler = Box<dyn Fn() -> Result<Response, Box<dyn std::error::Error>>>;

fn route(_method: &str, _path: &str) -> Option<Handler> {
    // Router implementation would go here
    None
}
