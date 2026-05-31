<script lang="ts">
	import { onMount } from 'svelte';
	import { handleLocaleStorageChange, initLocale } from '$lib/i18n/locale.svelte.js';

	onMount(() => {
		initLocale();
		window.addEventListener('storage', handleLocaleStorageChange);

		return () => {
			window.removeEventListener('storage', handleLocaleStorageChange);
		};
	});
</script>
