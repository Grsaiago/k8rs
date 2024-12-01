# K8rs: A k8s operator study in rust 🦀🦀

This operator listens for all cluster events, filters events concerning pods,
logs these events, and stores metrics that are then exported to Prometheus by
an http server that the controller runs concurrently on a 'green thread'.
Metrics include information such as the Pod ID and the server's UTC time of the event.

## Features

- Event Monitoring: Captures all Kubernetes cluster events.
- Pod Filtering: Focuses on pod-specific events.
- Logging: Logs pod events using the tracing crate.
- Metrics Exporting: Stores metrics with the `metrics` crate and
exposes them to prometheus via a `/metrics` endpoint using an
`Axum` server with `axum_prometheus_exporter`.

## Prerequisites

- A running Kubernetes cluster, or a local cluster created with Kind.
Rust and Cargo installed on your system. You can install them via rustup.

### Install dependencies

- Kind install
- Rust install

### Clone the repository

``` bash
git clone <repository-url>
cd <repository-directory>
```

## See it in action

### Start your kind cluster (optional)

```sh
kind create cluster --config kind-cluster.yml
```

### Build and run the operator

You can optionally build a release binary version of the operator:

``` sh
cargo build --release
```
<!--# TODO: BOTAR A PARADA DE IR ATÉ O BINÁRIO E EXECUTAR-->

But this should work as well (but it'll be a debug build):

``` sh
cargo run
```

### Apply a simple pod

```sh
kubectl apply -f manifest.yaml
```


Whenever you run

```sh
kubectl apply -f manifest.yaml
```

or

```sh
kubectl apply -f manifest.yaml
```

you should see the operator logging!
If you see the logs, check out your `localhost:8080/metrics` and you should see
the metrics being correctly exported to Prometheus
