<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import { Select as SelectPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { cn, type WithoutChildrenOrChild } from '$lib/utils.js';

	let {
		ref = $bindable(null),
		class: className,
		children: childrenProp,
		...restProps
	}: WithoutChildrenOrChild<SelectPrimitive.ItemProps> & {
		children?: Snippet;
	} = $props();
</script>

<SelectPrimitive.Item
	bind:ref
	data-slot="select-item"
	class={cn(
		'relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 pr-8 pl-2 text-sm outline-none select-none focus:bg-accent focus:text-accent-foreground data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:pointer-events-none data-disabled:opacity-50',
		className
	)}
	{...restProps}
>
	{#snippet children({ selected })}
		<span class="absolute right-2 flex size-3.5 items-center justify-center">
			{#if selected}
				<CheckIcon class="size-4" />
			{/if}
		</span>
		{@render childrenProp?.()}
	{/snippet}
</SelectPrimitive.Item>
