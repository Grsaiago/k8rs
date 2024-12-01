use axum::{routing::get, Router};
use axum_prometheus::{
    metrics::{counter, describe_counter, Unit},
    PrometheusMetricLayerBuilder,
};
use futures::StreamExt;
use k8s_openapi::api::core::v1::Event;
use kube::{
    runtime::{
        watcher::{self, watcher, Config},
        WatchStreamExt,
    },
    Api, Client, ResourceExt,
};
use std::error::Error;
use tokio::{net::TcpListener, task};
use tracing::{error, info};

// the names for our counters
const POD_DELETE_COUNTER: &str = "deleted_pods";
const POD_CREATE_COUNTER: &str = "created_pods";

// the names for our labels
const TIME_METRIC_LABEL: &str = "event_time";
const POD_ID_LABEL: &str = "pod_id";

// a struct for our metrics label
struct EventLabels {
    pub time: String,
    pub object_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize tracing for cool and shinny log.
    tracing_subscriber::fmt().init();

    // we'll initialize both the axum server socket and the k8s client first,
    // because if one of those fails, we souldn't do nothing else
    let listener = match TcpListener::bind("0.0.0.0:8080").await {
        Ok(conn) => conn,
        Err(err) => {
            error!("{:?}", err);
            return Err(Box::from(err));
        }
    };
    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(err) => {
            error!("{:?}", err);
            return Err(Box::from(err));
        }
    };

    // Initialize our counters with metadata
    // *Those are the counters that'll be exported to prometheus
    initialize_counters();

    // we'll spin up an http axum server to talk to prometheus
    task::spawn(async {
        // using axum-prometheus to crete the prometheus metrics exporter
        let (prom_layer, prom_handler) = PrometheusMetricLayerBuilder::new()
            .with_prefix("pods_operator")
            .with_default_metrics()
            .with_ignore_patterns(&["/ping", "/metrics", "/favicon.ico"]) // to reduce noise
            .build_pair();

        // create the axum router
        let app = Router::new()
            .route("/metrics", get(|| async move { prom_handler.render() }))
            .route("/ping", get(|| async move { "pong" })) // a healthcheck
            .layer(prom_layer);

        // serve the constructed router on the created socket
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await;
    });

    // this is the main routine, where we'll observe events and filter them into
    // only what we want to listen.
    task::spawn(async move {
        // we create a serializable way of communicating over a specific resource.
        // In this case, all events that happen on the cluster's "default" namespace.
        let pods: Api<Event> = Api::<Event>::default_namespaced(client.clone());

        // we pin the stream for 'async rust' reasons
        let mut event_stream = Box::pin(watcher(pods.clone(), Config::default()).default_backoff());

        loop {
            if let Some(event) = event_stream.next().await {
                // this match is kinda self explanatory
                match event {
                    Ok(watcher::Event::Apply(event)) | Ok(watcher::Event::Delete(event))
                        if event
                            .involved_object
                            .kind
                            .as_ref()
                            .is_some_and(|kind| kind == "Pod") =>
                    {
                        if let Some(ref reason) = event.reason {
                            match reason.as_ref() {
                                "Pulled" => info!("image for Pod {} pulled", event.name_any()),
                                "Created" => {
                                    let labels = extract_label_values_from_event(&event);
                                    // we get the counter with our labels and increment it
                                    counter!(
                                        POD_CREATE_COUNTER,
                                        &[
                                            (TIME_METRIC_LABEL, labels.time),
                                            (POD_ID_LABEL, labels.object_id)
                                        ]
                                    )
                                    .increment(1);
                                    info!("Pod {} created", event.name_any());
                                }
                                "Scheduled" => {
                                    info!("Pod {} scheduled", event.name_any())
                                }
                                "Started" => {
                                    info!("Pod {} allocated and started", event.name_any())
                                }
                                "Updated" => info!("Pod {} updated", event.name_any()),
                                "Killing" => {
                                    let labels = extract_label_values_from_event(&event);
                                    // we get the counter with our labels and increment it
                                    counter!(
                                        POD_DELETE_COUNTER,
                                        &[
                                            (TIME_METRIC_LABEL, labels.time),
                                            (POD_ID_LABEL, labels.object_id)
                                        ]
                                    )
                                    .increment(1);
                                    info!("Killing Pod {}", event.name_any());
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(watcher::Event::Init) => {
                        info!("Starting the watch stream...")
                    }
                    Ok(watcher::Event::InitDone) => {
                        info!("Watch stream up and running!")
                    }
                    Ok(_) => {} // we're not interested in init apply
                    Err(err) => {
                        error!("Error on receiving update: {:?}", err);
                    }
                }
            }
        }
    });

    // This is just a cancelation point for the operator.
    // It waits for a kill signal
    let _ = tokio::signal::ctrl_c().await;
    info!("Kill signal received, stopping...");

    Ok(())
}

fn initialize_counters() {
    describe_counter!(
        POD_DELETE_COUNTER,
        Unit::Count,
        "The number of deleted pods"
    );
    describe_counter!(
        POD_CREATE_COUNTER,
        Unit::Count,
        "The number of created pods"
    );
}

fn extract_label_values_from_event(ev: &Event) -> EventLabels {
    let time = ev.first_timestamp.as_ref().map_or("".to_string(), |date| {
        date.0
            .to_rfc3339_opts(k8s_openapi::chrono::SecondsFormat::Millis, false)
    });
    let object_id = ev
        .involved_object
        .uid
        .as_ref()
        .map_or("".to_string(), |val| val.clone());

    EventLabels { time, object_id }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
