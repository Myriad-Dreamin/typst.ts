use std::path::Path;

use clap::Parser;
use log::info;
use tokio::net::TcpListener;
use typst_ts_remote_server::{
    utils::async_continue, ws::Session as WsSession, Opts, RunArgs, Subcommands,
};

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let opts = Opts::parse();

    match opts.sub {
        Subcommands::Run(args) => run(args),
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn run(args: RunArgs) -> ! {
    let root = args.root.clone();
    let addr = args.web_socket;

    let root = Path::new(&root).canonicalize().unwrap();

    async_continue(async move {
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        info!("Listening on: {}", listener.local_addr().unwrap());

        while let Ok((stream, _)) = listener.accept().await {
            let addr = stream
                .peer_addr()
                .expect("connected streams should have a peer address");
            info!("Peer address: {}", addr);

            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
            info!("New WebSocket connection: {}", addr);

            let session = WsSession::over_tcp(&root, ws_stream);
            tokio::spawn(async move { session.serve().await });
        }
    });
}
