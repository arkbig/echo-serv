use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body, Body, Request, Response, Server, Uri};
use serde::Serialize;
use serde_json::{json, to_string_pretty, Value};
use signal_hook::low_level::exit;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAX_HISTORY: usize = 10;

#[derive(Debug, Serialize)]
struct AppState {
    requests: Vec<Value>,
}

fn get_url_string(uri: &Uri) -> String {
    let scheme = uri
        .scheme_str()
        .map(|s| (format!("{s}://")))
        .unwrap_or_default();
    let authority = uri.authority().map(|a| format!("{a}/")).unwrap_or_default();
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();

    format!("{scheme}{authority}{path}{query}")
}

fn get_version_str(version: &hyper::http::Version) -> &str {
    match version {
        &hyper::http::Version::HTTP_09 => "HTTP/0.9",
        &hyper::http::Version::HTTP_10 => "HTTP/1.0",
        &hyper::http::Version::HTTP_11 => "HTTP/1.1",
        &hyper::http::Version::HTTP_2 => "HTTP/2.0",
        &hyper::http::Version::HTTP_3 => "HTTP/3.0",
        _ => "Unknown",
    }
}

fn headers_to_json(headers: &hyper::HeaderMap) -> serde_json::Value {
    let mut json_obj = serde_json::Map::new();

    for (name, value) in headers {
        let name_str = name.as_str().to_owned();
        let value_str = value.to_str().unwrap_or("").to_owned();
        json_obj.insert(name_str, json!(value_str));
    }

    json!(json_obj)
}

async fn body_into_url_encoded(req: Request<Body>) -> anyhow::Result<String> {
    let body_bytes = body::to_bytes(req.into_body()).await?;
    let mut encoded = String::new();
    for &byte in body_bytes.iter() {
        match byte {
            // URLエンコードが必要な文字を指定する
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", byte));
            }
        }
    }
    Ok(encoded)
}

fn build_json_response(value: &Value) -> Response<Body> {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(value).unwrap()))
        .unwrap()
}

async fn handle_request(
    remote_addr: String,
    local_addr: String,
    req: Request<Body>,
    state: Arc<Mutex<AppState>>,
) -> anyhow::Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => {
            let state = state.lock().await;
            let history_json = to_string_pretty(&state.requests).unwrap();
            Ok(Response::new(Body::from(history_json)))
        }
        _ => {
            let value = json!({
                "client_addr": remote_addr,
                "server_addr": local_addr,
                "url": get_url_string(req.uri()),
                "method": req.method().as_str(),
                "version": get_version_str(&req.version()),
                "headers": headers_to_json(req.headers()),
                "body": body_into_url_encoded(req).await?,
            });
            let mut state = state.lock().await;
            state.requests.push(value.clone());
            if MAX_HISTORY < state.requests.len() {
                state.requests.remove(0);
            }
            println!("{}", value);
            Ok(build_json_response(&value))
        }
    }
}

#[tokio::main]
async fn main() {
    {
        use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
        let mut signals = signal_hook::iterator::Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGTERM])
            .expect("Error setting signal handler");
        std::thread::spawn(move || {
            for sig in signals.forever() {
                println!("Received signal {:?}", sig);
                exit(1);
            }
        });
    }

    let app_state = Arc::new(Mutex::new(AppState {
        requests: Vec::with_capacity(MAX_HISTORY),
    }));

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let state = app_state.clone();
        let remote_addr = conn.remote_addr().to_string();
        let local_addr = conn.local_addr().to_string();
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(
                    remote_addr.to_owned(),
                    local_addr.to_owned(),
                    req,
                    state.clone(),
                )
            }))
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 7878));
    let server = Server::bind(&addr).serve(make_svc);

    println!("Server listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
