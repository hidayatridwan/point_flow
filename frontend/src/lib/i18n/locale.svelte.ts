import { browser } from '$app/environment';
import { isLocale, messages, type Locale } from './messages.js';

const STORAGE_KEY = 'locale';
const DEFAULT_LOCALE: Locale = 'id';

export const localeState = $state<{ locale: Locale }>({
	locale: DEFAULT_LOCALE
});

export function initLocale() {
	if (!browser) return;

	setLocale(getStoredLocale() ?? DEFAULT_LOCALE);
}

export function getLocale() {
	return localeState.locale;
}

export function setLocale(locale: Locale) {
	applyLocale(locale, true);
}

export function toggleLocale() {
	setLocale(localeState.locale === 'id' ? 'en' : 'id');
}

export function handleLocaleStorageChange(event: StorageEvent) {
	if (event.key !== STORAGE_KEY) return;
	applyLocale(isLocale(event.newValue) ? event.newValue : DEFAULT_LOCALE, false);
}

export function t(path: string) {
	const value = path.split('.').reduce<unknown>((currentValue, key) => {
		if (!currentValue || typeof currentValue !== 'object') return undefined;
		return (currentValue as Record<string, unknown>)[key];
	}, messages[localeState.locale]);

	return typeof value === 'string' ? value : path;
}

function applyLocale(locale: Locale, shouldPersist: boolean) {
	localeState.locale = locale;

	if (!browser) return;

	if (shouldPersist) {
		window.localStorage.setItem(STORAGE_KEY, locale);
	}
	document.documentElement.lang = locale;
}

function getStoredLocale() {
	const value = window.localStorage.getItem(STORAGE_KEY);
	return isLocale(value) ? value : null;
}
