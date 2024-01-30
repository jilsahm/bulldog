use std::{collections::HashMap, time::{Duration, Instant}};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{client::DogClient, span::DogSpan};

pub struct DogShipper {
    client: DogClient,
    rx: Receiver<DogSpan>,
    traces: HashMap<u64, Vec<DogSpan>>,
    last_shippment: Instant,
}

impl DogShipper {

    pub fn new(client: DogClient) -> (Self, Sender<DogSpan>) {
        let (tx, rx) = channel(1024);
        (Self { client, rx, traces: HashMap::default(), last_shippment: Instant::now() }, tx)
    }

    pub async fn run(mut self) {
        loop {            
            match tokio::time::timeout(Duration::from_secs(5), self.rx.recv()).await {
                Ok(Some(span)) => self.traces.entry(span.trace_id).or_default().push(span),
                Ok(None) => return,
                Err(_) => (),
            }
            self.ship().await;
        }
    }

    async fn ship(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_shippment) > Duration::from_secs(5) && !self.traces.is_empty() {
            let mut traces = Vec::new();
            for (_, trace) in self.traces.drain() {
                traces.push(trace);
            }
            self.client.flush(&traces).await;
            self.last_shippment = now;            
        }
    }
}