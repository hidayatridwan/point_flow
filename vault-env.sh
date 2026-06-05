#!/usr/bin/env bash
# Usage: source ./vault-env.sh
# Exports all secrets from kv/pos/config into the current shell session.

VAULT_ADDR="${VAULT_ADDR:-http://127.0.0.1:8200}"
export VAULT_ADDR

if [[ -z "${VAULT_TOKEN:-}" ]]; then
  echo "error: VAULT_TOKEN is not set" >&2
  return 1
fi

eval "$(vault kv get -format=json -mount=kv/pos config \
  | jq -r '.data.data | to_entries[] | "export \(.key)=\(.value | @sh)"')"

echo "==> Secrets loaded from Vault."
