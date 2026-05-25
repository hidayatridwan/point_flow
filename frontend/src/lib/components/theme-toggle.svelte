<script lang="ts">
	import { browser } from '$app/environment';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	const THEME_STORAGE_KEY = 'theme';
	const DARK_THEME = 'dark';
	const LIGHT_THEME = 'light';
	const THEME_QUERY = '(prefers-color-scheme: dark)';

	type Theme = typeof DARK_THEME | typeof LIGHT_THEME;

	let isDark = $state(browser ? document.documentElement.classList.contains(DARK_THEME) : false);
	let mediaQuery: MediaQueryList | null = null;

	const nextTheme = $derived<Theme>(isDark ? LIGHT_THEME : DARK_THEME);
	const label = $derived(isDark ? t('theme.switchToLight') : t('theme.switchToDark'));

	onMount(() => {
		mediaQuery = window.matchMedia(THEME_QUERY);
		applyStoredOrSystemTheme();

		mediaQuery.addEventListener('change', handleSystemThemeChange);
		window.addEventListener('storage', handleStorageChange);

		return () => {
			mediaQuery?.removeEventListener('change', handleSystemThemeChange);
			window.removeEventListener('storage', handleStorageChange);
		};
	});

	function toggleTheme() {
		setTheme(nextTheme);
	}

	function setTheme(theme: Theme) {
		window.localStorage.setItem(THEME_STORAGE_KEY, theme);
		applyTheme(theme);
	}

	function applyStoredOrSystemTheme() {
		const storedTheme = getStoredTheme();
		applyTheme(storedTheme ?? getSystemTheme());
	}

	function handleSystemThemeChange() {
		if (getStoredTheme()) return;
		applyTheme(getSystemTheme());
	}

	function handleStorageChange(event: StorageEvent) {
		if (event.key !== THEME_STORAGE_KEY) return;
		applyStoredOrSystemTheme();
	}

	function getStoredTheme(): Theme | null {
		const value = window.localStorage.getItem(THEME_STORAGE_KEY);
		return value === DARK_THEME || value === LIGHT_THEME ? value : null;
	}

	function getSystemTheme(): Theme {
		return mediaQuery?.matches ? DARK_THEME : LIGHT_THEME;
	}

	function applyTheme(theme: Theme) {
		document.documentElement.classList.toggle(DARK_THEME, theme === DARK_THEME);
		isDark = theme === DARK_THEME;
	}
</script>

<Button type="button" variant="ghost" size="icon-sm" aria-label={label} onclick={toggleTheme}>
	{#if isDark}
		<SunIcon aria-hidden="true" />
	{:else}
		<MoonIcon aria-hidden="true" />
	{/if}
	<span class="sr-only">{label}</span>
</Button>
