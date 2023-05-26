use std::path::{Path, PathBuf};

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use typst_ts_core::config::CompileOpts;

use typst_ts_compiler::{service::CompileSession, world::WorldSnapshot};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum Event {
    Initialize(InitializeEvent),
    Compile(CompileEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeEvent {
    pub workspace: String,
    pub entry: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileEvent {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum EventResponse {
    WorldSnapshot(WorldSnapshotResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldSnapshotResponse {
    snapshot: Option<WorldSnapshot>,
    id: String,
}

pub struct Session {
    default_root: PathBuf,
    compile_opts: CompileOpts,
    pub tx: Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>,
    pub rx: Mutex<SplitStream<WebSocketStream<TcpStream>>>,
    pub compile_session: Mutex<CompileSession>,
}

impl Session {
    pub fn over_tcp(
        root: &Path,
        compile_opts: CompileOpts,
        ws_stream: WebSocketStream<TcpStream>,
    ) -> Self {
        let (tx, rx) = ws_stream.split();
        Self {
            default_root: root.to_owned(),
            compile_opts,
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
            compile_session: Default::default(),
        }
    }

    pub async fn serve(self) {
        let mut rx = self.rx.lock().await;

        while let Some(msg) = rx.next().await {
            info!("Received a message from client: {:?}", msg);

            let msg = match msg {
                Ok(msg) => msg,
                Err(err) => {
                    error!("failed to receive from client: {}", err);
                    return;
                }
            };

            let text_msg = match msg {
                Message::Text(text) => text,
                _ => continue,
            };

            let event: Event = match serde_json::from_str(&text_msg) {
                Ok(event) => event,
                Err(err) => {
                    error!("failed to parse event from client: {:?}", err);
                    continue;
                }
            };

            self.recv_event(event).await;
        }
    }

    async fn recv_event(&self, event: Event) {
        match event {
            Event::Initialize(event) => self.recv_initialize_event(event).await,
            Event::Compile(event) => self.recv_compile_event(event).await,
        }
    }

    async fn recv_initialize_event(&self, event: InitializeEvent) {
        let workspace = Path::new(&event.workspace).canonicalize().unwrap();
        let entry = workspace.join(event.entry).canonicalize().unwrap();

        let mut session = self.compile_session.lock().await;

        let initialized = 'initialize_chk: {
            if !workspace.starts_with(&self.default_root) {
                error!("invalid workspace: {}", workspace.display());
                break 'initialize_chk false;
            }

            let base_compile_opts = self.compile_opts.clone();
            let compile_opts = CompileOpts {
                root_dir: workspace,
                ..base_compile_opts
            };

            session.initialize(entry, compile_opts)
        };

        let snapshot = initialized.then(|| session.take_snapshot()).flatten();

        drop(session);

        comemo::evict(30);

        self.send_world_snapshot(WorldSnapshotResponse {
            snapshot,
            id: event.id,
        })
        .await;
    }

    async fn recv_compile_event(&self, event: CompileEvent) {
        let mut session = self.compile_session.lock().await;

        let snapshot = session.take_snapshot();

        drop(session);

        // Garbage collect incremental cache. This evicts all memoized results that haven't been
        // used in the last 30 compilations.
        comemo::evict(30);

        self.send_world_snapshot(WorldSnapshotResponse {
            snapshot,
            id: event.id,
        })
        .await;
    }

    async fn send_world_snapshot(&self, response: WorldSnapshotResponse) {
        let msg = match serde_json::to_string(&EventResponse::WorldSnapshot(response)) {
            Ok(response) => Message::Text(response),
            Err(err) => {
                error!("failed to serialize WorldSnapshot: {:?}", err);
                return;
            }
        };

        let mut tx = self.tx.lock().await;
        tx.send(msg).await.unwrap();
    }
}
