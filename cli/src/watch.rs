use std::path::Path;

use log::{error, info};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

pub async fn watch_dir(
    workspace_dir: &Path,
    mut interrupted_by_events: impl FnMut(Option<Vec<Event>>),
) -> ! {
    // Setup file watching.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, _>| match res {
            Ok(e) => {
                tx.send(e).unwrap();
            }
            Err(e) => error!("watch error: {:#}", e),
        },
        notify::Config::default(),
    )
    .map_err(|_| "failed to watch directory")
    .unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(workspace_dir, RecursiveMode::Recursive)
        .unwrap();

    // Handle events.
    info!("start watching files...");
    interrupted_by_events(None);
    loop {
        let mut events = vec![];
        while let Ok(e) =
            tokio::time::timeout(tokio::time::Duration::from_millis(100), rx.recv()).await
        {
            if e.is_none() {
                continue;
            }
            events.push(e.unwrap());
        }

        interrupted_by_events(Some(events));
    }
}
