import { env } from '$env/dynamic/public';

declare global {
	interface Window {
		snap?: {
			pay: (
				token: string,
				callbacks?: {
					onSuccess?: (result: unknown) => void;
					onPending?: (result: unknown) => void;
					onError?: (result: unknown) => void;
					onClose?: () => void;
				}
			) => void;
		};
	}
}

const SANDBOX_SNAP_URL = 'https://app.sandbox.midtrans.com/snap/snap.js';
const PRODUCTION_SNAP_URL = 'https://app.midtrans.com/snap/snap.js';

export function getMidtransClientKey(): string {
	return env.PUBLIC_MIDTRANS_CLIENT_KEY ?? '';
}

export function isMidtransProduction(): boolean {
	return (env.PUBLIC_MIDTRANS_IS_PRODUCTION ?? 'false').toLowerCase() === 'true';
}

function snapScriptUrl(): string {
	return isMidtransProduction() ? PRODUCTION_SNAP_URL : SANDBOX_SNAP_URL;
}

let snapLoadPromise: Promise<void> | null = null;

/** Muat `snap.js` sekali (sandbox vs production via env). Gagal bila client key kosong. */
export function loadMidtransSnap(): Promise<void> {
	if (typeof document === 'undefined') return Promise.reject(new Error('No document'));
	if (window.snap) return Promise.resolve();
	if (snapLoadPromise) return snapLoadPromise;

	snapLoadPromise = new Promise<void>((resolve, reject) => {
		const clientKey = getMidtransClientKey();
		if (!clientKey) {
			reject(new Error('Missing Midtrans client key'));
			return;
		}
		const script = document.createElement('script');
		script.src = snapScriptUrl();
		script.setAttribute('data-client-key', clientKey);
		script.async = true;
		script.onload = () => {
			if (window.snap) resolve();
			else reject(new Error('Midtrans Snap failed to initialize'));
		};
		script.onerror = () => reject(new Error('Midtrans Snap failed to load'));
		document.head.appendChild(script);
	});
	snapLoadPromise.catch(() => {
		snapLoadPromise = null;
	});
	return snapLoadPromise;
}
