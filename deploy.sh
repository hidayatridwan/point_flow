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

COMPOSE="docker compose -f docker-compose.prod.yml"

# Manual, opt-in schema helpers. These are never run by the deploy path below;
# you invoke them by hand, one file at a time, e.g.:
#   ./deploy.sh migrate 001_initial_pos_schema.sql
#   ./deploy.sh seed dashboard_sample.sql
# (No migration tracking table exists, so apply files individually and in order.)
case "${1:-}" in
  migrate)
    [[ -n "${2:-}" ]] || { echo "usage: ./deploy.sh migrate <file.sql>" >&2; exit 1; }
    exec $COMPOSE exec -T postgres \
      psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -f "/sql/migrations/$2"
    ;;
  seed)
    [[ -n "${2:-}" ]] || { echo "usage: ./deploy.sh seed <file.sql>" >&2; exit 1; }
    exec $COMPOSE exec -T postgres \
      psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -f "/sql/seeds/$2"
    ;;
esac

# Pass any other subcommand directly to docker compose with secrets injected
if [[ $# -gt 0 ]]; then
  exec $COMPOSE "$@"
fi

# No args = full deploy
if [[ -n "${GHCR_TOKEN:-}" ]]; then
  echo "==> Logging in to GHCR..."
  echo "${GHCR_TOKEN}" | docker login ghcr.io -u hidayatridwan --password-stdin
fi

# The 'web' network is owned by the Caddy/infra stack — don't create it here,
# just fail clearly if it's missing.
docker network inspect web >/dev/null 2>&1 || {
  echo "error: external docker network 'web' not found (created by the Caddy/infra stack)" >&2
  exit 1
}

echo "==> Deploying POS stack..."
$COMPOSE pull
$COMPOSE up -d

echo "==> Done."
$COMPOSE ps
echo "==> Health check..."
curl -fsS http://127.0.0.1:${APP_PORT:-8081}/api/v1/health || true
