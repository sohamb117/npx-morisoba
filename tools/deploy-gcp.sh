#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${GCP_PROJECT:?Set GCP_PROJECT to the destination project ID}"
REGION="${GCP_REGION:-us-east1}"
SERVICE="${GCP_SERVICE:-morisoba}"
IMAGE="${REGION}-docker.pkg.dev/${GCP_PROJECT}/morisoba/web:$(date -u +%Y%m%d%H%M%S)"
bash "$ROOT/morisoba-wasm/build.sh"
gcloud builds submit "$ROOT/morisoba-wasm" --project="$GCP_PROJECT" --region="$REGION" --tag="$IMAGE"
gcloud run deploy "$SERVICE" --project="$GCP_PROJECT" --region="$REGION" \
    --image="$IMAGE" --port=8080 --allow-unauthenticated \
    --cpu=1 --memory=256Mi --min-instances=0 --max-instances=2 --quiet
