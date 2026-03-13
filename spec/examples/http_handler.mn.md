# HTTP Handler — Implementation

Implementation of a backend HTTP service with auth, routing, and error handling.

```monel
use std/http {Request, Response, Method, Handler, Route, Server, StatusCode}
use std/time {Duration, Instant, Clock}
use std/log {info, error}
use std/crypto {Crypto}

type Config
  port: Int
  host: String
  tls: Option<TlsConfig>
  max_connections: Int
  request_timeout: Duration

type ServerError
  | PortInUse
  | PermissionDenied
  | TlsError(String)

type AuthError
  | InvalidCreds
  | Locked
  | Expired

type Credentials
  username: String
  password: String

type Session
  user_id: Int
  token: String
  expires_at: Instant

fn serve @intent("serve")
  params: config: Config, handler: Handler
  returns: Result<Server, ServerError>
  effects: [Net.listen, Log.write]
  body:
    info("starting server", port: config.port, host: config.host)
    let server = try Server.bind(config.host, config.port)
    server.set_max_connections(config.max_connections)
    server.set_timeout(config.request_timeout)
    match config.tls
      | Some(tls) => try server.enable_tls(tls)
      | None => ()
    info("server listening", port: config.port)
    Ok(server.start(handler))

fn route @intent("route")
  params: method: Method, pattern: String, handler: Handler
  returns: Route
  effects: [pure]
  body:
    Route.new(method, pattern, handler)

fn authenticate @intent("authenticate")
  params: creds: Credentials
  returns: Result<Session, AuthError>
  effects: [Db.read, Db.write, Crypto.verify, Log.write]
  body:
    let user = Db.find_user_by_username(creds.username)
    match user
      | None =>
        info("auth_failed", reason: "user_not_found")
        Err(AuthError.InvalidCreds)
      | Some(u) =>
        if u.failed_attempts >= 5
          info("auth_blocked", reason: "account_locked", user_id: u.id)
          Err(AuthError.Locked)
        else
          let valid = Crypto.verify(creds.password, u.password_hash)
          if valid
            Db.reset_failed_attempts(u.id)
            let existing = Db.find_active_session(u.id)
            match existing
              | Some(s) if s.expires_at <= Clock.now() =>
                Db.delete_session(s.token)
                info("auth_failed", reason: "session_expired", user_id: u.id)
                Err(AuthError.Expired)
              | _ =>
                let session = Session
                  user_id: u.id
                  token: Crypto.generate_token()
                  expires_at: Clock.now().add(Duration.hours(24))
                Db.insert_session(session)
                info("auth_success", user_id: u.id)
                Ok(session)
          else
            Db.increment_failed_attempts(u.id)
            info("auth_failed", reason: "invalid_password", user_id: u.id)
            Err(AuthError.InvalidCreds)

fn handle_request @intent("handle_request")
  params: req: Request
  returns: Response
  effects: [Db.read, Db.write, Log.write, Http.send]
  body:
    let start = Clock.now()
    let response = match Router.match(req.method, req.path)
      | None =>
        Response.new(StatusCode.NotFound, "not found")
      | Some(route) =>
        let result = route.handler(req)
        match result
          | Ok(resp) => resp
          | Err(e) =>
            error("request_failed", path: req.path, error: e.to_string())
            Response.new(StatusCode.InternalServerError, "internal error")
    let elapsed = Clock.now().since(start)
    info("request_complete", method: req.method, path: req.path, status: response.status, duration_ms: elapsed.as_millis())
    response
```
