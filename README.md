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

#### Add Istio Sidecar Injection - Mutation policy

// TODO

#### Verify Image - Admission policy

This policy is defined in
[`kubernetes/local-ssi/kyverno/verify-image-signature.yml`](kubernetes/local-ssi/kyverno/verify-image-signature.yml).

After the container image is built [in the
CI](https://github.com/Kuruyia/ssi-kubernetes/blob/2196e44dff3e865d02c3c52d6820814080ecb5aa/.github/workflows/on_push_main.yml#L94-L105),
it is signed with [cosign](https://github.com/sigstore/cosign) using a private
key supplied as a GitHub Actions secret.

Kyverno then verifies the signature of all container images whose name begins
with `ghcr.io/kuruyia/ssi-kubernetes/` against the corresponding public key,
and rejects the pod if the signature is invalid.

**How to test:** You can modify the public key given to Kyverno to an invalid
one, and check that Kyverno prevents the pods from being run because they can
no longer be verified:

```sh
# In the `kubernetes/local-ssi/` directory
sed -i '' 's/MFkw/MFaa/g' kyverno/verify-image-signature.yml
kubectl apply -k .
kubectl delete pods -n ssi-kubernetes --all
```

Then, check the events in the app namespace with `kubectl events -n
ssi-kubernetes`:

```sh
[...]
2s (x15 over 90s)   Warning   FailedCreate       ReplicaSet/verbs-774478894f        Error creating: admission webhook "mutate.kyverno.svc-fail" denied the request: 

resource Pod/ssi-kubernetes/ was blocked due to the following policies 

verify-image:
  verify-image: 'failed to verify image ghcr.io/kuruyia/ssi-kubernetes/words:latest:
    .attestors[0].entries[0].keys: failed to load public key from PEM: pem to public
    key: asn1: structure error: tags don''t match (16 vs {class:2 tag:26 length:19
    isCompound:false}) {optional:false explicit:false application:false private:false
    defaultValue:<nil> tag:<nil> stringType:0 timeType:0 set:false omitEmpty:false}
    AlgorithmIdentifier @2'
```

As you can see, the pods are being blocked by Kyverno. You can also see that
there are no pods in the app namespace:

```sh
$ kubectl get pods -n ssi-kubernetes
No resources found in ssi-kubernetes namespace.
```

To revert the changes:

```sh
# In the `kubernetes/local-ssi/` directory
git restore .
kubectl apply -k .
```

#### Require resource requests - Admission policy

This policy is defined in
[`kubernetes/local-ssi/kyverno/require-requests.yml`](kubernetes/local-ssi/kyverno/require-requests.yml).

Any pod that contains container which do not set resource requests will be
prevented from running.

**How to test:** You can try to patch one of the deployments of this app to
remove the resource requests:

```sh
$ kubectl patch deploy aggregator -n ssi-kubernetes -p '{"spec":{"template":{"spec":{"containers":[{"name":"aggregator", "resources":{"requests":null}}]}}}}'
Error from server: admission webhook "validate.kyverno.svc-fail" denied the request: 

resource Deployment/ssi-kubernetes/aggregator was blocked due to the following policies 

require-requests:
  autogen-validate-resources: 'validation error: CPU and memory resource requests
    are required. rule autogen-validate-resources failed at path /spec/template/spec/containers/0/resources/requests/'
```

As you can see, Kyverno blocks the change due to the missing resource requests.

### Falco

// TODO

### Istio

// TODO
