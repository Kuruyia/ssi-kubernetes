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

#### Add Default securityContext - Mutation policy

This policy is defined in
[`kubernetes/local-ssi/kyverno/add-securitycontext.yml`](kubernetes/local-ssi/kyverno/add-securitycontext.yml).

This adds the following configuration on the pod security context:
- `runAsNonRoot: true`: requires the pod containers to run as non-root users
- `runAsUser: 1000`: runs container processes with user ID 1000
- `runAsGroup: 3000`: runs container processes with primary group ID 3000
- `fsGroup: 2000`: the owner group ID of files in mounted volumes is 2000

This adds the following configuration on the pod containers security context:
- `allowPrivilegeEscalation: false`: forbids processes to gain more privileges
  than their parent processes (for example, setuid/setgid binaries)
- `readOnlyRootFilesystem: true`: mounts the root container filesystem as
  read-only
- `capabilities: drop: - ALL`: drops all Linux capabilities (privileged
  operations)

Those are default values for the security context, and will _not_ override
values that would be defined in the resource itself.

**How to test:** You can make sure that the `securityContext` is set correctly
on one of the app pods:

```sh
$ kubectl get pods -n ssi-kubernetes -l 'app.kubernetes.io/name=aggregator' -o yaml | grep -A 4 securityContext
      securityContext:
        allowPrivilegeEscalation: false
        capabilities:
          drop:
          - ALL
--
    securityContext:
      fsGroup: 2000
      runAsGroup: 3000
      runAsNonRoot: true
      runAsUser: 1000
```

Alternatively, you can, for instance, make sure that the container of an app
deployment is running as the specified user/group:

```sh
$ kubectl exec deploy/aggregator -n ssi-kubernetes -- id
uid=1000 gid=3000 groups=3000,2000
```

### Falco

// TODO

### Istio

// TODO
