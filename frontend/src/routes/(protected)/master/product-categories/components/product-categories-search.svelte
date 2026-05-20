<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import { Input } from '$lib/components/ui/input/index.js';

	let {
		value,
		disabled = false,
		onValueChange
	}: {
		value: string;
		disabled?: boolean;
		onValueChange: (value: string) => void;
	} = $props();
</script>

<div class="relative w-full sm:max-w-xs">
	<SearchIcon
		class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
	/>
	<Input
		type="search"
		aria-label="Cari kategori produk"
		placeholder="Cari kategori produk..."
		class="pl-8"
		{value}
		{disabled}
		oninput={(event) => onValueChange(event.currentTarget.value)}
	/>
</div>
