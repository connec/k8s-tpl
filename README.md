# `kubetpl`

Templatisation for Kubernetes manifests

## Overview

`kubetpl` is a CLI tool for interpolating Kubernetes manifests using the Go templating language.
The supported input and output format is intended to facilitate usage in a pipeline with `kubectl apply -f -`.

## Usage

The CLI is largely self documenting:

```sh
kubetpl --help
```

### Basic usage

```sh
kubetpl --filename kubernetes.yaml --config dev.yaml \
  | kubectl apply -f -
```
