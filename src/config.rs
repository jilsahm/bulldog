use std::{env, ffi::OsStr, path::Path};

#[derive(Clone)]
pub struct DogConfig {
    // name of the service being traced
    pub service: String,
    // url of the datadog (cluster) agent trace API
    pub url: String,
}

impl Default for DogConfig {

    fn default() -> Self {
        Self {
            service: env::current_exe()
                .ok()
                .as_ref()
                .map(Path::new)
                .and_then(Path::file_name)
                .and_then(OsStr::to_str)
                .map(String::from)
                .expect("failed to get current executable name"),
            url: "http://localhost:8126/v0.3/traces".to_string() 
        }
    }
}