use hyper::client::conn as HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::Body, client, Request, Response, Server};
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;
use base64::encode;

#[derive(Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

async fn proxy_handler(
    req: Request<Body>,
    client: Client<HttpsConnector<HttpConnector>>,
    proxy_config: ProxyConfig,
) -> Result<Response<Body>, hyper::Error> {
    let auth_header = format!("Basic {}", encode(format!("{}:{}", 
        proxy_config.username, 
        proxy_config.password
    )));

    let mut proxy_req = Request::builder()
        .uri(req.uri().clone())
        .method(req.method().clone())
        .body(req.into_body())?;

    proxy_req.headers_mut().insert(
        "Proxy-Authorization",
        auth_header.parse().unwrap(),
    );

    Ok(client.request(proxy_req).await?)
}

pub async fn start_proxy_server(config: ProxyConfig) {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);
    let proxy_config = config.clone();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    
    let make_svc = make_service_fn(move |_| {
        let client = client.clone();
        let proxy_config = proxy_config.clone();
        
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                proxy_handler(req, client.clone(), proxy_config.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Proxy server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}