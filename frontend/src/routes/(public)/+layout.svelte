<script lang="ts">
	import { onMount } from 'svelte';
	import LocaleSwitch from '$lib/components/storefront/locale-switch.svelte';
	import { reloadCartFromStorage } from '$lib/cart.svelte.js';

	let { children } = $props();

	onMount(() => {
		reloadCartFromStorage();
		const onStorage = (event: StorageEvent) => {
			if (event.key === 'pointflow:cart:v1') reloadCartFromStorage();
		};
		window.addEventListener('storage', onStorage);
		return () => window.removeEventListener('storage', onStorage);
	});
</script>

<svelte:head>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Poppins:wght@400;600;700&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<div
	class="min-h-svh bg-neutral-100 text-[#333333]"
	style="font-family: 'Poppins', ui-sans-serif, system-ui, sans-serif;"
>
	<div class="pointer-events-none fixed top-4 right-4 z-20">
		<div class="pointer-events-auto">
			<LocaleSwitch />
		</div>
	</div>
	{@render children()}
</div>
