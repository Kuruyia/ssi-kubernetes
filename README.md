# SSI - Kubernetes security

This project is a deliverable for the SSI course at Polytech Montpellier, 5th
year of DO.

## Deploy to k3d

Deployment of this project has been tested on a Kubernetes cluster managed by
k3d.

### Prerequisites

First, [install k3d](https://k3d.io/stable/#installation) and spin up a new
cluster with:

```sh
k3d cluster create -p "8081:80@loadbalancer"
```

This will create a single-node local cluster that maps `localhost:8081` to the
port 80 of the load balancer in the cluster. If port 8081 is already used on
your host machine, you can change it to another port.

Then, multiple services will need to be installed in the cluster as part of the
security hardening:

- [Kyverno](https://kyverno.io/docs/installation/methods/)
- [Falco](https://falco.org/docs/getting-started/falco-kubernetes-quickstart/)
- [Istio](https://istio.io/latest/docs/setup/install/helm/) 

In a nutshell:

```sh
# Kyverno
helm repo add kyverno https://kyverno.github.io/kyverno/
helm repo update
helm install kyverno kyverno/kyverno -n kyverno --create-namespace

# Falco
# TODO

# Istio
# TODO
```

### Deploy the project

The project will be deployed to the cluster using
[kustomize](https://kustomize.io/).

To deploy this project to your prepared k3d cluster, simply go to the
[`kubernetes/local-ssi/`](./kubernetes/local-ssi) directory in a shell and run
`kubectl apply -k .`.

If everything went well, you should be able to request the aggregator service
by running `curl http://localhost:8081/`. You should see a JSON response with a
`sentence` key containing a random sentence.

## Security measures

### Kyverno

// TODO

### Falco

// TODO

### Istio

// TODO
