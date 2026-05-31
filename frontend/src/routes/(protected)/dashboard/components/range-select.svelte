<script lang="ts">
	import * as Select from '$lib/components/ui/select/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		value = $bindable(30),
		disabled = false,
		onChange
	}: {
		value?: number;
		disabled?: boolean;
		onChange?: (days: number) => void;
	} = $props();

	const options = $derived([
		{ value: '7', label: t('dashboard.charts.last7Days') },
		{ value: '30', label: t('dashboard.charts.last30Days') },
		{ value: '90', label: t('dashboard.charts.last90Days') }
	]);

	const selectedLabel = $derived(
		options.find((option) => option.value === String(value))?.label ?? ''
	);
</script>

<Select.Root
	type="single"
	allowDeselect={false}
	value={String(value)}
	{disabled}
	onValueChange={(next) => {
		if (!next) return;
		value = Number(next);
		onChange?.(value);
	}}
>
	<Select.Trigger class="h-8 w-[150px] text-xs" aria-label={t('dashboard.charts.rangeLabel')}>
		<Select.Value placeholder={selectedLabel}>{selectedLabel}</Select.Value>
	</Select.Trigger>
	<Select.Content>
		{#each options as option (option.value)}
			<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
		{/each}
	</Select.Content>
</Select.Root>
