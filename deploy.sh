#!/usr/bin/env bash
set -euo pipefail

VAULT_ADDR="${VAULT_ADDR:-http://127.0.0.1:8200}"
export VAULT_ADDR

if [[ -z "${VAULT_TOKEN:-}" ]]; then
  echo "error: VAULT_TOKEN is not set" >&2
  exit 1
fi

echo "==> Fetching secrets from Vault (kv/pos/config)..."
eval "$(vault kv get -format=json -mount=kv/pos config \
  | jq -r '.data.data | to_entries[] | "export \(.key)=\(.value | @sh)"')"

if [[ -n "${GHCR_TOKEN:-}" ]]; then
  echo "==> Logging in to GHCR..."
  echo "${GHCR_TOKEN}" | docker login ghcr.io -u hidayatridwan --password-stdin
fi

docker network inspect proxy >/dev/null 2>&1 || docker network create proxy

echo "==> Deploying POS stack..."
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d

echo "==> Done."
docker compose -f docker-compose.prod.yml ps
echo "==> Health check..."
curl -fsS http://127.0.0.1:${APP_PORT:-8081}/api/v1/health || true
