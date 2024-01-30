use std::{collections::HashMap, sync::{Mutex, RwLock}, time::SystemTime};

use tokio::{runtime::{Builder, Runtime}, sync::mpsc::Sender};
use tracing::{field::Visit, span::{self, Id}, Subscriber};
use tracing_subscriber::Layer;

use crate::{client::DogClient, config::DogConfig, shipper::DogShipper, span::DogSpan};

pub struct DogTracingLayer {
    config: DogConfig,
    spans: RwLock<HashMap<span::Id, Mutex<DogSpan>>>,
    runtime: Runtime,
    tx: Sender<DogSpan>,
}

struct V {
    dd_trace_id: u64,
}

impl V {

    pub fn new() -> Self {
        Self {
            dd_trace_id: rand::random(),
        }
    }
}

impl Visit for V {
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "dd_trace_id" {
            self.dd_trace_id = value;
        }
    }

    fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {
        // no-op
    }
}

impl DogTracingLayer {

    pub fn new(config: DogConfig, client: DogClient) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("valid runtime creation");

        let (shipper, tx) = DogShipper::new(client);

        runtime.spawn(async move {
            shipper.run().await;
        });

        Self {
            config,
            spans: RwLock::default(),
            runtime,
            tx,
        }
    }
}

impl <S: Subscriber> Layer<S> for DogTracingLayer {

    fn on_new_span(&self, attributes: &span::Attributes<'_>, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let trace_id = match attributes.parent() {
            None => {
                let mut visitor = V::new();
                attributes.values().record(&mut visitor);
                visitor.dd_trace_id
            },
            Some(parent) => {
                let read_lock = self.spans.read().expect("valid read lock for spans");
                if let Some(mtx) = read_lock.get(parent) {
                    let lock = mtx.lock().expect("valid write lock on span");
                    lock.trace_id
                } else {
                    let mut visitor = V::new();
                    attributes.values().record(&mut visitor);
                    visitor.dd_trace_id
                }            
            },
        };

        let dog_span = DogSpan::new(
            attributes.metadata().name(),
            attributes.parent().map(Id::into_u64),
            "", // todo: make resource field settable
            &self.config.service,
            id.into_u64(),
            0,
            trace_id,
        );

        let mut lock = self.spans.write().expect("valid spans lock");
        lock.insert(id.clone(), Mutex::new(dog_span));

    }

    fn on_enter(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let read_lock = self.spans.read().expect("valid read lock for spans");
        if let Some(mtx) = read_lock.get(id) {
            let mut write_lock = mtx.lock().expect("valid write lock on span");
            write_lock.start = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
        }
    }

    fn on_exit(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mtx = {
            let mut write_lock = self.spans.write().expect("valid write lock for spans");
            write_lock.remove(id)
        };
        if let Some(mtx) = mtx {
            let mut span = mtx.into_inner().expect("valid span");
            let tx = self.tx.clone();

            span.duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos() as u64 - span.start;
            self.runtime.spawn(async move { tx.send(span).await.expect("bug in dog subscriber"); });
        }
    }
}