use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::process::exit;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use typst::diag::SourceResult;
use typst::geom::{Abs, Size};
use typst::World;
use typst_ts_core::artifact::core::EcoString;
use typst_ts_core::artifact::doc::Frame;
use typst_ts_core::artifact::font::FontInfo;
use typst_ts_core::{Artifact, ArtifactExporter};

#[derive(Debug)]
struct Client {
    tx: Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>,
    rx: Mutex<SplitStream<WebSocketStream<TcpStream>>>,
    visible_range: Mutex<Option<Range<usize>>>,
}

impl Client {
    async fn send(&self, artifact: Arc<Artifact>, new_meta: bool) -> bool {
        #[derive(Debug, Serialize)]
        struct Info {
            page_total: usize,
            page_numbers: Vec<usize>,
            width: u32,
            height: u32,
            fonts: Vec<FontInfo>,
            title: Option<EcoString>,
            author: Vec<EcoString>,
            pages: Vec<Frame>,
        }
        let visible_range = self
            .visible_range
            .lock()
            .await
            .clone()
            .unwrap_or(0..artifact.pages.len());
        let mut pages_to_send = vec![];
        {
            for i in visible_range.clone() {
                if i < artifact.pages.len() {
                    pages_to_send.push(i);
                }
            }
        }
        if pages_to_send.is_empty() && !new_meta {
            return true;
        }

        let size = if !artifact.pages.is_empty() {
            artifact.pages[0].size.into()
        } else {
            Size::new(Abs::raw(0.), Abs::raw(0.))
        };

        let info = serde_json::to_string(&Info {
            page_total: artifact.pages.len(),
            page_numbers: pages_to_send.clone(),
            width: size.x.to_pt() as u32,
            height: size.y.to_pt() as u32,
            fonts: artifact.fonts.clone(),
            title: artifact.title.clone(),
            author: artifact.author.clone(),
            pages: if new_meta {
                artifact.pages.clone()
            } else {
                vec![]
            },
        })
        .unwrap();

        if let Err(err) = self.tx.lock().await.send(Message::Text(info)).await {
            error!("failed to send to client: {}", err);
            return false;
        }
        for i in visible_range.clone() {
            let _ = self
                .tx
                .lock()
                .await
                .send(Message::Text(
                    serde_json::to_string(&artifact.pages[i]).unwrap(),
                ))
                .await; // don't care result here
        }
        info!("visible range {:?}", visible_range);
        true
    }

    async fn poll_visible_range(&self, artifact: Arc<Mutex<Option<Arc<Artifact>>>>) -> bool {
        while let Some(msg) = self.rx.lock().await.next().await {
            #[derive(Debug, Deserialize)]
            struct VisibleRangeInfo {
                page_start: usize,
                page_end: usize,
            }
            match msg {
                Ok(msg) => {
                    if let Message::Text(text) = msg {
                        let range: Result<VisibleRangeInfo, _> = serde_json::from_str(&text);
                        if let Err(range) = range {
                            error!("failed to parse visible range: {:?}", range);
                            continue;
                        }
                        let range: VisibleRangeInfo = range.unwrap();
                        *self.visible_range.lock().await = Some(range.page_start..range.page_end);
                        if let Some(artifact) = artifact.lock().await.clone() {
                            self.send(artifact, false).await;
                        }
                    }
                }
                Err(err) => {
                    error!("failed to receive from client: {}", err);
                    return false;
                }
            }
        }
        error!("client disconnected");
        exit(-1);
    }
}

#[derive(Clone)]
pub struct WebSocketArtifactExporter {
    conns: Arc<Mutex<Vec<Arc<Client>>>>,
    current_artifact: Arc<Mutex<Option<Arc<Artifact>>>>,
}

impl WebSocketArtifactExporter {
    pub fn new_url(addr: String) -> Self {
        let this = Self {
            conns: Arc::new(Mutex::new(Vec::new())),
            current_artifact: Arc::new(Mutex::new(None)),
        };

        let remote_this = this.clone();
        tokio::spawn(async move {
            // Create the event loop and TCP listener we'll accept connections on.
            let try_socket = TcpListener::bind(&addr).await;
            let listener = try_socket.expect("Failed to bind");
            info!("Listening on: {}", listener.local_addr().unwrap());

            while let Ok((stream, _)) = listener.accept().await {
                let conn = accept_connection(stream).await;
                {
                    let (tx, rx) = conn.split();
                    let client = Arc::new(Client {
                        tx: Mutex::new(tx),
                        rx: Mutex::new(rx),
                        visible_range: Mutex::new(None),
                    });
                    remote_this.conns.lock().await.push(client.clone());
                    {
                        let current_artifact = remote_this.current_artifact.clone();
                        tokio::spawn(async move {
                            if let Some(artifact) = current_artifact.lock().await.clone() {
                                client.send(artifact, true).await;
                            }
                            client.poll_visible_range(current_artifact).await
                        });
                    }
                }
            }
        });

        this
    }
}

impl ArtifactExporter for WebSocketArtifactExporter {
    /// Export the given artifact with given world.
    fn export(&self, _world: &dyn World, output: Arc<Artifact>) -> SourceResult<()> {
        let remote_this = self.clone();
        tokio::spawn(async move {
            *remote_this.current_artifact.lock().await = Some(output.clone());
            broadcast_result(remote_this.conns, output).await;
        });
        Ok(())
    }
}

async fn accept_connection(stream: TcpStream) -> WebSocketStream<TcpStream> {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);
    ws_stream
}

async fn broadcast_result(
    conns: Arc<Mutex<Vec<Arc<Client>>>>,
    artifact: Arc<Artifact>,
    // hashes: Vec<u128>,
) {
    let mut conn_lock = conns.lock().await;
    info!("render done, sending to {} clients", conn_lock.len());
    let mut to_be_remove: Vec<usize> = vec![];
    for (i, conn) in conn_lock.iter_mut().enumerate() {
        if !conn.send(artifact.clone(), true).await {
            to_be_remove.push(i);
        }
    }
    // remove
    conn_lock.retain(with_index(|index, _item| !to_be_remove.contains(&index)));
}

fn with_index<T, F>(mut f: F) -> impl FnMut(&T) -> bool
where
    F: FnMut(usize, &T) -> bool,
{
    let mut i = 0;
    move |item| (f(i, item), i += 1).0
}
