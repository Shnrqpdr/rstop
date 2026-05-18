use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::metrics::{Collectors, Histories, Snapshot};

pub struct AppState {
    pub snapshot: Snapshot,
    pub history: Histories,
    pub config: Config,
    pub paused: bool,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let history = Histories::new(config.history);
        Self {
            snapshot: Snapshot::default(),
            history,
            config,
            paused: false,
        }
    }
}

pub type SharedState = Arc<RwLock<AppState>>;

pub fn spawn_poller(state: SharedState) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut collectors = Collectors::new();
        let rt = tokio::runtime::Handle::current();

        let initial_interval = rt.block_on(async {
            let s = state.read().await;
            s.config.interval
        });
        std::thread::sleep(initial_interval);

        loop {
            let (interval, paused) = rt.block_on(async {
                let s = state.read().await;
                (s.config.interval, s.paused)
            });

            if !paused {
                collectors.refresh();
                let snap = Snapshot {
                    cpu: collectors.cpu(),
                    mem: collectors.mem(),
                    temp: collectors.temp(),
                    gpus: collectors.gpu(),
                };

                rt.block_on(async {
                    let mut s = state.write().await;
                    s.history.push(&snap);
                    s.snapshot = snap;
                });
            }

            std::thread::sleep(interval);
        }
    })
}

pub fn render_interval() -> Duration {
    Duration::from_millis(100)
}
