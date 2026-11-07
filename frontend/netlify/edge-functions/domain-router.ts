/**
 * Router per-domain untuk satu situs Netlify yang sama.
 *
 * Satu situs (satu build SPA) melayani dua peran:
 *   - Host yang terdaftar di env `SHOP_DOMAINS` = storefront publik `/shop*`.
 *   - Host lain (mis. `pointflow.netlify.app`, deploy preview) = dashboard kasir.
 *
 * Selama `SHOP_DOMAINS` belum di-set di Netlify (scope "Functions"), fungsi
 * ini no-op: semua host berperilaku kasir seperti sebelumnya. Env edge
 * function harus di-set di dashboard Netlify (bukan netlify.toml) dan
 * perubahan nilainya berlaku setelah deploy baru (dokumentasi Netlify).
 */

/** Host dashboard default (domain Netlify bawaan); bisa dioverride via `KASIR_DOMAINS`. */
const KASIR_FALLBACK_HOST = 'pointflow.netlify.app';

/**
 * Prefix route dashboard — mengikuti group `(protected)` di
 * `frontend/src/routes`. Kalau ada modul dashboard baru dengan prefix lain,
 * tambahkan di sini. Path di luar daftar ini (termasuk `/_app/*` dan static
 * assets) tidak pernah diarahkan ke host lain.
 */
const DASHBOARD_PREFIXES = [
	'/account',
	'/config',
	'/dashboard',
	'/help-desk',
	'/inventory',
	'/master',
	'/payroll',
	'/point_flow',
	'/procurement',
	'/reports'
] as const;

// Global `Netlify` tersedia di runtime edge function Netlify; dideklarasikan
// lokal supaya file ini tidak butuh dependency `@netlify/edge-functions`.
declare const Netlify: { env: { get(key: string): string | undefined } };

interface EdgeContext {
	next: (request?: Request) => Promise<Response>;
}

const normalizeHost = (host: string): string =>
	host
		.trim()
		.toLowerCase()
		.replace(/^www\./, '');

const parseDomains = (raw: string | undefined): string[] =>
	(raw ?? '').split(',').map(normalizeHost).filter(Boolean);

const redirect = (location: string, status: 301 | 302): Response =>
	new Response(null, { status, headers: { location } });

const isDashboardPath = (pathname: string): boolean =>
	DASHBOARD_PREFIXES.some((prefix) => pathname === prefix || pathname.startsWith(`${prefix}/`));

export default async function domainRouter(
	request: Request,
	context: EdgeContext
): Promise<Response> {
	const url = new URL(request.url);
	const pathWithQuery = `${url.pathname}${url.search}`;
	const shopHosts = parseDomains(Netlify.env.get('SHOP_DOMAINS'));
	const kasirHost = parseDomains(Netlify.env.get('KASIR_DOMAINS'))[0] ?? KASIR_FALLBACK_HOST;
	const shopOrigin = shopHosts.length > 0 ? `https://${shopHosts[0]}` : null;

	if (shopHosts.includes(normalizeHost(url.hostname))) {
		// Host storefront: root langsung ke /shop, halaman dashboard diarahkan ke host kasir.
		if (url.pathname === '/') {
			return redirect(`${url.origin}/shop${url.search}`, 302);
		}
		if (isDashboardPath(url.pathname)) {
			return redirect(`https://${kasirHost}${pathWithQuery}`, 301);
		}
		return context.next();
	}

	// Host kasir / host lain: /shop* milik storefront di domain utama.
	if (shopOrigin && (url.pathname === '/shop' || url.pathname.startsWith('/shop/'))) {
		return redirect(`${shopOrigin}${pathWithQuery}`, 301);
	}
	return context.next();
}

export const config = { path: '/*' };
