# simkube

A collection of tools for simulating Kubernetes scheduling and autoscaling behaviour

## Overview

This package provides the following:

- `simkube`: a [Virtual Kubelet](https://virtual-kubelet.io)-based "hollow node" that allows customization based off a
  "skeleton" node file (see the example in `simkube/manifests/dist/0000-simkube.k8s.yaml`)
- `sk-cloudprov`: an [external gRPC-based cloud provider](https://github.com/kubernetes/autoscaler/tree/master/cluster-autoscaler/cloudprovider/externalgrpc)
  for Cluster Autoscaler that can communicate with and scale the `simkube` "node group".  An example configuration
  for `sk-cloudprov` and Cluster Autoscaler can be found in `simkube/manifests/dist/0002-sk-cloudprov.k8s.yaml` and
  `simkube/manifests/dist/0003-cluster-autoscaler.k8s.yaml`.

## Monitoring

We use the [kube-prometheus](https://github.com/prometheus-operator/kube-prometheus/tree/main) stack to set up
prometheus and grafana for monitoring and data collection.  You need to install `jsonnet`, using your system package
manager or otherwise.

## Developing

It is highly recommended that you install [pre-commit](https://pre-commit.com); this will run useful checks before you
push anything to GitHub.  To set up the hooks in this repo, run `pre-commit install`.  You will also need to install
[go-carpet](https://github.com/msoap/go-carpet) 1.11.0 or higher:

```
go install https://github.com/msoap/go-carpet@latest
```

You can develop and test locally against a [kind](https://kind.sigs.k8s.io) cluster.  First, create your kind cluster:

```
make kind
```

You only need to do the above step once unles you change something about your cluster configuration.  To deploy
`simkube` and `sk-cloudprov`, run

```
make build image
kubectl apply -f manifests/dist
```

This will also create a test deployment which is scheduled on the virtual nodes.  If you scale the test deployment up or
down, Cluster Autoscaler and sk-cloudprov will react to scale the `simkube` deployment object.
