use reqwest::Client;

use crate::{config::DogConfig, span::DogSpan};


pub struct DogClient {
    config: DogConfig,
    inner: Client,
}

unsafe impl Send for DogClient{}

impl DogClient {

    pub fn new(config: DogConfig) -> Self {
        let inner = reqwest::ClientBuilder::new().build().expect("valid http client creation");
        Self {
            config,
            inner,
        }
    }

    pub async fn flush(&self, traces: &Vec<Vec<DogSpan>>) {
        debug!("flushing {:?}", traces.len());
        match self.inner.put(&self.config.url)
            .json(traces)
            .send()
            .await 
        {
            Ok(resp) => debug!("agent response: {}", resp.status()),
            Err(e) => warn!("failed to flush traces: {}", e),
        }
    }
}