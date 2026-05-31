<script lang="ts">
	import { onMount } from 'svelte';
	import * as Table from '$lib/components/ui/table/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { getLowStock, type LowStockItem } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatNumber } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';

	let items = $state<LowStockItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const hasData = $derived(items.length > 0);

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = null;
		try {
			items = await getLowStock(8);
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			items = [];
		} finally {
			loading = false;
		}
	}
</script>

<ChartCard
	title={t('dashboard.charts.lowStockTitle')}
	description={t('dashboard.charts.lowStockDesc')}
	{loading}
	{error}
	empty={!hasData}
	emptyText={t('dashboard.charts.lowStockEmpty')}
>
	<div class="overflow-x-auto">
		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head>{t('dashboard.charts.colProduct')}</Table.Head>
					<Table.Head>{t('dashboard.charts.colCategory')}</Table.Head>
					<Table.Head class="text-right">{t('dashboard.charts.colStock')}</Table.Head>
					<Table.Head class="text-right">{t('dashboard.charts.colMinimum')}</Table.Head>
					<Table.Head class="text-right">{t('dashboard.charts.colOnOrder')}</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each items as item (item.productSkuId)}
					<Table.Row class="transition-colors hover:bg-muted/60">
						<Table.Cell>
							<div class="font-medium">{item.productName}</div>
							<div class="text-xs text-muted-foreground">{item.sku}</div>
						</Table.Cell>
						<Table.Cell class="text-muted-foreground">{item.categoryName ?? '-'}</Table.Cell>
						<Table.Cell class="text-right">
							<Badge variant="destructive" class="font-mono tabular-nums">
								{formatNumber(item.stock)}
								{item.unit}
							</Badge>
						</Table.Cell>
						<Table.Cell class="text-right font-mono text-muted-foreground tabular-nums">
							{formatNumber(item.minimumStock)}
						</Table.Cell>
						<Table.Cell class="text-right font-mono text-muted-foreground tabular-nums">
							{formatNumber(item.quantityOnOrder)}
						</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
	</div>
</ChartCard>
