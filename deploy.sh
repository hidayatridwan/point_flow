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

echo "==> Deploying POS stack..."
docker compose -f docker-compose.prod.yml up -d --build

echo "==> Done."
docker compose -f docker-compose.prod.yml ps
