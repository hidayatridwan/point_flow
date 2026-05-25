<script lang="ts">
	import { onMount } from 'svelte';
	import { handleLocaleStorageChange, initLocale } from '$lib/i18n/locale.svelte.js';
	import LanguageToggle from './language-toggle.svelte';
	import ThemeToggle from './theme-toggle.svelte';

	onMount(() => {
		initLocale();
		window.addEventListener('storage', handleLocaleStorageChange);

		return () => {
			window.removeEventListener('storage', handleLocaleStorageChange);
		};
	});
</script>

<div class="fixed top-4 right-4 z-50 flex items-center gap-1">
	<LanguageToggle />
	<ThemeToggle />
</div>
