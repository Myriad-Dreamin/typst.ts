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

    let corpora = warp::path("corpus").and(warp::fs::dir(args.corpus));
    let assets = warp::path("assets").and(warp::fs::dir("assets"));
    let core = warp::path("core").and(warp::fs::dir("packages/typst.ts"));

    // map these files to the root of the github-pages server
    let gh_pages = warp::path("typst.ts").and({
        let renderer = warp::path("renderer").and(warp::fs::dir("packages/renderer/pkg"));
        let compiler = warp::path("compiler").and(warp::fs::dir("packages/compiler/pkg"));
        let typst_main =
            warp::path("typst-main.js").and(warp::fs::file("packages/typst.ts/dist/main.js"));

        renderer
            .or(compiler)
            .or(typst_main)
            .or(warp::fs::dir("github-pages"))
    });

    let cors =
        warp::cors().allow_methods(&[Method::HEAD, Method::GET, Method::POST, Method::DELETE]);

    let routes = corpora
        .or(assets)
        .or(core)
        .or(gh_pages)
        .with(cors)
        .with(warp::compression::gzip());

    let server = warp::serve(routes);

    server.run(http_addr).await
}
