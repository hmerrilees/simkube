TRACE_PATH="${TRACE_PATH:-file:///$HOME/data/trace}"

sh scripts/registry.sh # drmorr's script in troubleshooting

# === get skctl ===
make skctl

# === Set up kind cluster, sk-tracer + sk-ctrl  ===
kind create cluster --name simkube --config scripts/kind.yml
kubectl apply -k k8s/kustomize/sim

# === Installs ===
# kwok
KWOK_REPO=kubernetes-sigs/kwok
KWOK_LATEST_RELEASE=$(curl "https://api.github.com/repos/${KWOK_REPO}/releases/latest" | jq -r '.tag_name')
kubectl apply -f "https://github.com/${KWOK_REPO}/releases/download/${KWOK_LATEST_RELEASE}/kwok.yaml"
# prometheus
git clone https://github.com/prometheus-operator/kube-prometheus.git
cd kube-prometheus
kubectl create -f manifests/setup
until kubectl get servicemonitors --all-namespaces ; do date; sleep 1; echo ""; done
kubectl create -f manifests/
cd ..
# cert manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.3/cert-manager.yaml
kubectl wait --for=condition=Ready -l app=webhook -n cert-manager pod --timeout=60s
kubectl apply -f scripts/self-signed.yml

# === Export Trace ===
export POD_NAME="pod/$(kubectl get pods --all-namespaces -l app.kubernetes.io/name=sk-tracer -o custom-columns=NAME:.metadata.name --no-headers)"
# wait for pod to be ready
kubectl wait --for=condition=Ready "$POD_NAME" -n simkube
kubectl port-forward -n simkube "$POD_NAME" 7777:7777 & 
# get trace
./.build/skctl export -o "$TRACE_PATH" 

# === Run simulation ===
./.build/skctl run my-simulation --trace-path "$TRACE_PATH"
kubectl get simulations
