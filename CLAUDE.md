# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

POS Web System — a web-based cashier application with a Rust/Axum backend and a SvelteKit frontend. The two are developed independently; the backend is the source of truth for business logic and database schema.

## Commands

### Backend (run from repo root)

```bash
cargo run --manifest-path backend/Cargo.toml --bin server   # start server
cargo fmt --manifest-path backend/Cargo.toml                # format
cargo clippy --manifest-path backend/Cargo.toml --all-targets
cargo test --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml <test_name>   # single test
```

### Frontend (run from `frontend/`)

```bash
bun run dev       # dev server at http://localhost:5173
bun run check     # svelte-check + tsc
bun run lint      # prettier + eslint
bun run build
bun run format    # auto-fix formatting
```

### Infrastructure

```bash
docker compose up --build              # backend + postgres
docker compose up -d postgres          # postgres only (for local backend dev)
curl http://localhost:8080/api/v1/health
```

Backend reads `.env` from the **repo root**. Copy `.env.example` to `.env` before first run.

## Architecture

### Backend (`backend/src/`)

**Entry**: `bin/server.rs` → `app.rs` (AppState, middleware) → `http/mod.rs` (root router).

Each domain lives in `modules/<name>/` with a fixed internal structure:
- `routes.rs` — Axum route registration
- `handler.rs` — HTTP handler functions (extract request, call service, return JSON)
- `service.rs` — business logic
- `repository.rs` — SQLx database queries
- `model.rs` — domain structs
- `dto.rs` — request/response shapes

**Cross-cutting services** in `common/services/` are shared across modules:
- `master.rs` — product categories, products, units, SKUs, suppliers, employees
- `documents.rs` — purchase orders, goods receipts, stock adjustments/conversions
- `cash_registers.rs` — register sessions, cash drawer movements
- `auth.rs` — login, refresh, user management
- `reports.rs` — dashboard and all report queries
- `rbac.rs` — role/permission checks
- `audit.rs` — audit log writes

**Response envelope**: all API responses use `ApiResponse<T>` which the frontend unwraps automatically. Pagination uses `PaginationMeta` (`page`, `limit`, `total_records`, `total_pages`).

**Auth model**: JWT access token (short-lived, sent as `Authorization: Bearer`) + HttpOnly cookie refresh token. The backend has an RBAC system via `roles` table — the `admin` role is protected and cannot be assigned via user CRUD.

**Inventory model**: `product_skus` is the stock-holding unit. Stock changes only from:
- goods receipt post (adds stock)
- stock adjustment post (sets stock to corrected value)
- stock conversion post (moves stock between SKUs of same product)
- completed sale (reduces stock)

**Document lifecycle**: procurement and inventory documents all follow: `draft` → action (ordered/posted) → `cancelled` (terminal). Cancellation is only possible while the related register session is open for cash movements.

**Migrations** are in `backend/migrations/` as plain SQL files numbered sequentially. Docker Compose mounts these into `/docker-entrypoint-initdb.d` for first-run init.

### Frontend (`frontend/src/`)

**Framework**: SvelteKit 2 with Svelte 5 runes (`$state`, `$derived`, `$props`, `$effect`). All component state uses runes, not stores.

**Routing**: file-based. Protected routes are under `routes/(protected)/` with a layout that checks the access token (via `refreshAccessToken` on mount) and redirects to `/` if unauthenticated.

**API layer** (`lib/api/`):
- `client.ts` — `apiGet`, `apiPost`, `apiPut`, `apiDelete` wrappers. Handles: auth header injection, automatic token refresh on 401 (single in-flight refresh promise), JSON envelope unwrapping, `ApiError` throwing.
- `auth.svelte.ts` — `authState` is a Svelte 5 `$state` holding the in-memory access token. Cleared on logout; persists only for the session tab.
- One file per domain (e.g. `products.ts`, `purchase-orders.ts`) — each exports typed async functions that call the client wrappers.
- `PUBLIC_API_BASE_URL` env var overrides the default `http://localhost:8080/api/v1`.

**UI components** (`lib/components/`):
- `ui/` — shadcn-svelte primitives (Button, Dialog, Select, Table, etc.)
- Feature components (PageHeader, SearchField, DataPagination, ErrorAlert, StatusBadge, DetailField, SessionTransition) are shared across routes.
- Tables use TanStack Table Core via `createSvelteTable` + `createColumnHelper`.
- Forms use `sveltekit-superforms` + `zod` for validation.
- Toasts use `svelte-sonner`.

**i18n** (`lib/i18n/`):
- `messages.ts` — all UI strings for `id` (Indonesian, default) and `en`. Nested key structure mirroring the module hierarchy (`login.*`, `dashboard.*`, `master.employees.*`, `procurement.purchaseOrders.*`, etc.).
- `locale.svelte.ts` — `localeState` is a `$state` holding the active locale. `t(path)` does dot-notation traversal of `messages[locale]`. Locale is persisted to `localStorage` and synced across tabs. Status badge labels that must react to locale changes are defined as `$derived` records calling `t()`.
- Language toggle (EN/ID) is in `app-controls.svelte` (fixed top-right).
- DB data (product names, employee names, transaction numbers) is never translated — only UI chrome.

**Formatters** (`lib/formatters.ts`): `formatDate`, `formatDateTime`, `formatIdr`, `formatNumber` — use `Intl` APIs. Currency is always IDR regardless of locale.

**Theme**: dark/light toggle stored in `localStorage`, applied via `document.documentElement.classList`. Initialized by inline script in `app.html` to avoid flash.

### Key Conventions

- All new page-level components follow: `onMount` triggers data load, `$state` for data/loading/error, `$derived` for computed values, `ErrorAlert` for load failures.
- CRUD pages: list page owns pagination state and search debounce (350ms). Dialogs receive data via props and emit `onSaved`/`onDeleted`/`onCreated` callbacks.
- All UI strings must use `t()` from `$lib/i18n/locale.svelte.js` — never hardcode user-visible text.
- Zod schemas for form validation are module-scope constants (not reactive); validation messages remain in English.
- The `cash_register_session` is a prerequisite for POS transactions and cash movements — always check for an active session before enabling these flows.
