<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import { Input } from '$lib/components/ui/input/index.js';
	import { cn } from '$lib/utils.js';

	let {
		value,
		disabled = false,
		placeholder = 'Search...',
		ariaLabel = 'Search',
		class: className,
		onValueChange
	}: {
		value: string;
		disabled?: boolean;
		placeholder?: string;
		ariaLabel?: string;
		class?: string;
		onValueChange: (value: string) => void;
	} = $props();
</script>

<div class={cn('relative w-full sm:max-w-xs', className)}>
	<SearchIcon
		class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
	/>
	<Input
		type="search"
		aria-label={ariaLabel}
		{placeholder}
		class="pl-8"
		{value}
		{disabled}
		oninput={(event) => onValueChange(event.currentTarget.value)}
	/>
</div>
