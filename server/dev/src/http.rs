use std::net::SocketAddr;

use warp::Filter;

use crate::RunHttpArgs;

pub async fn run_http(args: RunHttpArgs) {
    use warp::http::Method;

    let mut http_addr = args.http.clone();
    if http_addr.is_empty() {
        http_addr = "127.0.0.1:20810".to_owned();
    }
    let http_addr: SocketAddr = http_addr.parse().unwrap();

    let corpora = warp::path("corpus")
        .and(warp::fs::dir(args.corpus))
        .with(warp::compression::gzip());
    let cors = warp::cors().allow_methods(&[Method::GET, Method::POST, Method::DELETE]);

    let routes = corpora.with(cors);

    let server = warp::serve(routes);

    server.run(http_addr).await
}
