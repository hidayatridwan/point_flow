import { t } from './locale.svelte.js';

/**
 * Translate a message key and interpolate `{placeholder}` values, e.g.
 * `tf('validation.required', { field: t('common.name') })`.
 */
export function tf(key: string, vars: Record<string, string | number> = {}) {
	let message = t(key);
	for (const [name, value] of Object.entries(vars)) {
		message = message.replace(`{${name}}`, String(value));
	}
	return message;
}
