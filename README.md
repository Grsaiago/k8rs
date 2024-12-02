# K8rs: A k8s operator study in rust 🦀🦀

This operator listens for all cluster events, filters events concerning pods,
logs these events, and stores metrics that are then exported to Prometheus by
an http server that the controller runs concurrently on a 'green thread'.
Metrics include information such as the Pod ID and the server's UTC time of the event.

## Features

- Event Monitoring: Captures all Kubernetes cluster events.
- Pod Filtering: Focuses on pod-specific events.
- Logging: Logs pod events using the `tracing` crate.
- Metrics Exporting: Stores metrics with the `metrics` crate and
exposes them to prometheus via a `/metrics` endpoint using an
`Axum` server with `axum_prometheus_exporter`.

## Prerequisites

- A running Kubernetes cluster, or a local cluster created with Kind.
- Rust and Cargo installed on your system.

### Install dependencies

- [Kind install](https://kind.sigs.k8s.io/docs/user/quick-start#installation)
- [Rust install](https://www.rust-lang.org/tools/install)
- [Kubectl install](https://kubernetes.io/docs/tasks/tools/#kubectl)

### Clone the repository

``` bash
git clone https://github.com/Grsaiago/k8rs/tree/main <where_you_want_to_clone>
cd <where_you_want_to_clone>
```

## See it in action

### Start your kind cluster

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

## Example

Now, whenever you run or delete a pod:

<div>
<img src="https://github.com/user-attachments/assets/688c51b8-d944-41a5-bf28-ac63c486a906" height=500 />
</div>

You should see the following in the Operator's console:

<div>
<img src="https://github.com/user-attachments/assets/0cc91f79-19e6-449b-8a48-2128a4c4e3ba" height=500 />
</div>


If you see the logs, don't forget to check out your `localhost:8080/metrics` to see
the metrics being correctly exported to Prometheus.
