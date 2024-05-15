use std::net::SocketAddr;

use warp::Filter;

use crate::RunHttpArgs;

/// See: <https://fasterthanli.me/articles/why-is-my-rust-build-so-slow>
pub async fn run_http(args: RunHttpArgs) {
    use warp::http::Method;

    let mut http_addr = args.http.clone();
    if http_addr.is_empty() {
        "127.0.0.1:20810".clone_into(&mut http_addr);
    }
    let http_addr: SocketAddr = http_addr.parse().unwrap();

    let root = (warp::path::end().or(warp::path("index.html")))
        .map(|_| warp::redirect(warp::http::Uri::from_static("/core/index.html")))
        .boxed();
    let corpora = warp::path("corpus").and(warp::fs::dir(args.corpus)).boxed();
    let assets = warp::path("assets").and(warp::fs::dir("assets")).boxed();
    let core = warp::path("core")
        .and(warp::fs::dir("packages/typst.ts"))
        .boxed();
    let base = warp::path("base").and(warp::fs::dir("")).boxed();
    let ets = warp::path("enhanced-typst-svg")
        .and(warp::fs::dir("packages/enhanced-typst-svg"))
        .boxed();

    // map these files to the root of the github-pages server
    let gh_pages = warp::path("typst.ts").and({
        let renderer = warp::path("renderer")
            .and(warp::fs::dir("packages/renderer/pkg"))
            .boxed();
        let compiler = warp::path("compiler")
            .and(warp::fs::dir("packages/compiler/pkg"))
            .boxed();
        let typst_main = warp::path("typst-main.js")
            .and(warp::fs::file("packages/typst.ts/dist/esm/main.bundle.js"))
            .boxed();

        renderer
            .or(compiler)
            .or(typst_main)
            .or(warp::fs::dir("github-pages"))
    });

    let cors =
        warp::cors().allow_methods(&[Method::HEAD, Method::GET, Method::POST, Method::DELETE]);

    let routes = root
        .or(corpora)
        .or(assets)
        .or(core)
        .or(base)
        .or(gh_pages)
        .or(ets)
        .with(cors)
        .with(warp::compression::gzip());

    let server = warp::serve(routes);

    server.run(http_addr).await
}
