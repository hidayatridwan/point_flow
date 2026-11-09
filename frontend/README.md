# Point Flow Frontend

Aplikasi kasir + storefront Point Flow: SvelteKit 2 + Svelte 5 (runes), Tailwind CSS 4, shadcn-svelte, TanStack Table, superforms+zod, svelte-sonner. Dokumentasi pengembangan ada di root [`CLAUDE.md`](../CLAUDE.md).

- Rute kasir di `src/routes/(protected)/` (butuh login, cek token saat mount).
- Storefront publik di `src/routes/(public)/shop` (katalog, checkout guest/member).
- API client di `src/lib/api/` (`client.ts` unwrapping envelope + auto-refresh 401, satu file per domain).
- i18n ID/EN di `src/lib/i18n/` (`t()` + `localeState`, persisten di `localStorage`).
- Mata uang selalu IDR (`src/lib/formatters.ts`).

## Developing

```bash
bun install
bun run dev      # http://localhost:5173
```

Backend default `http://localhost:8080/api/v1`; override dengan env `PUBLIC_API_BASE_URL`.

## Checks & build

```bash
bun run check    # svelte-check + tsc
bun run lint     # prettier + eslint
bun run build
bun run format   # auto-fix formatting
```
