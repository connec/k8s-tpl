# `k8s-tpl`

Templatisation for Kubernetes manifests

## Overview

`k8s-tpl` is a CLI tool for interpolating Kubernetes manifests using the Go templating language.
The supported input and output format is intended to facilitate usage in a pipeline with `kubectl apply -f -`.

## Installation

The `k8s-tpl` CLI is available on [crates.io](https://crates.io/) and can be installed using Cargo:

```sh
cargo install k8s-tpl
```

## Usage

The CLI is largely self documenting:

```sh
k8s-tpl --help
```

### Basic usage

```sh
k8s-tpl --filename kubernetes.yaml --config dev.yaml \
  | kubectl apply -f -
```

### Environment variables

`k8s-tpl` injects environment variables into an `Env` key in the template context.
