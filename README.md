# Bulldog
An experimental [tracing](https://crates.io/crates/tracing) layer for sending traces to a Datadog agent via the Datadog tracing API.

In case your datadog agents have the OTel endpoints enabled, just use the [OTel](https://crates.io/crates/tracing-opentelemetry) tracing exporter instead.

## Usage
Setup the following components with your configuration. It is highly recommended have the agents run as cluster agents in order to
reach the tracing API via localhost.
```[rust]
let dd_config = DogConfig {
    service: "sample-application".to_string(),
    url: "http://localhost:8126/v0.3/traces".to_string(),
};
let dd_client = DogClient::new(dd_config.clone());
let dd_layer = DogTracingLayer::new(dd_config, dd_client);
```

Mount the layer onto your prefered tracing subscriber.
```[rust]
let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::TRACE)
    .finish()
    .with(dd_layer);

tracing::subscriber::set_global_default(subscriber)
    .expect("setting default subscriber failed");
```