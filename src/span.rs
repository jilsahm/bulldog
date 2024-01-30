use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum SpanType {
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "db")]
    Database,
    #[serde(rename = "cache")]
    Cache,
    #[serde(rename = "custom")]
    Custom,
}


#[derive(Debug, Serialize)]
pub struct DogSpan {

    /// The duration of the request in nanoseconds.
    pub duration: u64,

    /// Set this value to 1 to indicate if an error occurred. If an error occurs, you should pass additional information, 
    /// such as the error message, type and stack information in the meta property.
    pub error: u8,

    /// A set of key-value metadata. Keys and values must be strings.
    pub meta: HashMap<String, String>,

    /// A set of key-value metadata. Keys must be strings and values must be 64-bit floating point numbers.
    pub metrics: HashMap<String, f64>,

    /// The span name. The span name must not be longer than 100 characters.
    pub name: String,

    /// The span integer ID of the parent span.
    pub parent_id: Option<u64>,

    /// The resource you are tracing. The resource name must not be longer than 5000 characters.
    pub resource: String,

    /// The service you are tracing. The service name must not be longer than 100 characters.
    pub service: String,

    /// The span integer (64-bit unsigned) ID.
    pub span_id: u64,

    /// The start time of the request in nanoseconds from the UNIX epoch.
    pub start: u64,

    /// The unique integer (64-bit unsigned) ID of the trace containing this span.
    pub trace_id: u64,

    /// The type of request. Allowed enum values: web, db, cache, custom
    pub r#type: SpanType,
}

unsafe impl Send for DogSpan{}
unsafe impl Sync for DogSpan{}

impl DogSpan {

    pub fn new(
        name: &str,
        parent_id: Option<u64>,
        resource: &str,
        service: &str,
        span_id: u64,
        start: u64,
        trace_id: u64,
    ) -> Self {
        Self {
            duration: 0,
            error: 0,
            meta: HashMap::default(),
            metrics: HashMap::default(),
            name: name.to_string(),
            parent_id,
            resource: resource.to_string(),
            service: service.to_string(),
            span_id,
            start,
            trace_id,
            r#type: SpanType::Custom,
        }
    }
}