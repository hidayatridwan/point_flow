<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import CirclePlusIcon from '@lucide/svelte/icons/circle-plus';
	import type { Component } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Command from '$lib/components/ui/command/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { cn } from '$lib/utils.js';

	type FacetedFilterOption = {
		label: string;
		value: string;
		icon?: Component;
	};

	let {
		title,
		options,
		value = [],
		disabled = false,
		onValueChange
	}: {
		title: string;
		options: FacetedFilterOption[];
		value?: string[];
		disabled?: boolean;
		onValueChange?: (value: string[]) => void;
	} = $props();

	const selectedValues = $derived(new SvelteSet(value));

	function toggleValue(optionValue: string) {
		const nextValues = new SvelteSet(value);

		if (nextValues.has(optionValue)) {
			nextValues.delete(optionValue);
		} else {
			nextValues.add(optionValue);
		}

		onValueChange?.(Array.from(nextValues));
	}

	function clearFilters() {
		onValueChange?.([]);
	}
</script>

<Popover.Root>
	<Popover.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="outline" size="sm" class="h-9 border-dashed" {disabled}>
				<CirclePlusIcon />
				{title}
				{#if selectedValues.size > 0}
					<Separator orientation="vertical" class="mx-1 h-4" />
					<Badge variant="secondary" class="rounded-sm px-1 font-normal lg:hidden">
						{selectedValues.size}
					</Badge>
					<div class="hidden gap-1 lg:flex">
						{#if selectedValues.size > 2}
							<Badge variant="secondary" class="rounded-sm px-1 font-normal">
								{t('common.selectedCount').replace('{count}', String(selectedValues.size))}
							</Badge>
						{:else}
							{#each options.filter( (option) => selectedValues.has(option.value) ) as option (option.value)}
								<Badge variant="secondary" class="rounded-sm px-1 font-normal">
									{option.label}
								</Badge>
							{/each}
						{/if}
					</div>
				{/if}
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-56 p-0" align="start">
		<Command.Root>
			<Command.Input placeholder={`${t('common.filter')} ${title.toLowerCase()}...`} />
			<Command.List>
				<Command.Empty>{t('common.noResults')}</Command.Empty>
				<Command.Group>
					{#each options as option (option.value)}
						{@const isSelected = selectedValues.has(option.value)}
						<Command.Item onSelect={() => toggleValue(option.value)}>
							<div
								class={cn(
									'me-2 flex size-4 items-center justify-center rounded-sm border border-primary',
									isSelected ? 'bg-primary text-primary-foreground' : 'opacity-50 [&_svg]:invisible'
								)}
							>
								<CheckIcon class="size-4" />
							</div>
							{#if option.icon}
								{@const Icon = option.icon}
								<Icon class="text-muted-foreground" />
							{/if}
							<span>{option.label}</span>
						</Command.Item>
					{/each}
				</Command.Group>
				{#if selectedValues.size > 0}
					<Command.Separator />
					<Command.Group>
						<Command.Item onSelect={clearFilters} class="justify-center text-center">
							{t('common.clear')}
						</Command.Item>
					</Command.Group>
				{/if}
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
