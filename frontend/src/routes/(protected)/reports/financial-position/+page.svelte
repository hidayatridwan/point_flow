<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import {
		getFinancialPosition,
		type FinancialPosition,
		type FinancialPositionParams
	} from '$lib/api/reports.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import Calendar from '$lib/components/ui/calendar/calendar.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDate, formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let position = $state<FinancialPosition | null>(null);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<FinancialPositionParams>>(initialQuery);
	let asOfValue = $state<DateValue | undefined>(dateStringToDateValue(initialQuery.as_of));
	let isAsOfOpen = $state(false);
	let latestRequestId = 0;

	const rows = $derived(
		position
			? [
					{
						label: t('reports.financialPosition.cash'),
						value: position.cash,
						strong: false
					},
					{
						label: t('reports.financialPosition.inventoryValue'),
						value: position.inventory_value,
						strong: false
					},
					{
						label: t('reports.financialPosition.totalAssets'),
						value: position.total_assets,
						strong: true
					},
					{
						label: t('reports.financialPosition.cumulativeNetProfit'),
						value: position.cumulative_net_profit,
						strong: true
					}
				]
			: []
	);

	onMount(() => {
		void loadPosition(query);
	});

	$effect(() => {
		const as_of = asOfValue?.toString() ?? '';
		if (as_of === query.as_of) return;

		updateAsOfFilter(as_of);
	});

	async function loadPosition(params: FinancialPositionParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getFinancialPosition(params);
			if (requestId !== latestRequestId) return;
			position = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			position = null;
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function updateAsOfFilter(as_of: string) {
		query = { as_of };
		syncQueryUrl(query);
		void loadPosition(query);
	}

	function clearAsOfFilter() {
		asOfValue = undefined;
		isAsOfOpen = false;
	}

	function parseQuery(searchParams: URLSearchParams): Required<FinancialPositionParams> {
		return { as_of: searchParams.get('as_of') ?? '' };
	}

	function syncQueryUrl(nextQuery: Required<FinancialPositionParams>) {
		const nextUrl = new URL(page.url);
		setOptionalSearchParam(nextUrl, 'as_of', nextQuery.as_of);
		replaceState(resolve(`${nextUrl.pathname}${nextUrl.search}` as '/'), page.state);
	}

	function setOptionalSearchParam(url: URL, key: string, value: string) {
		if (value) {
			url.searchParams.set(key, value);
		} else {
			url.searchParams.delete(key);
		}
	}

	function dateStringToDateValue(value: string): DateValue | undefined {
		if (!value) return undefined;

		try {
			return parseDate(value);
		} catch {
			return undefined;
		}
	}

	function getAsOfLabel() {
		if (position && !query.as_of)
			return `${t('reports.financialPosition.asOf')} ${formatDate(position.as_of)}`;
		if (query.as_of) return `${t('reports.financialPosition.asOf')} ${formatDate(query.as_of)}`;
		return t('reports.financialPosition.selectDate');
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('reports.financialPosition.errorLoad');
	}
</script>

<svelte:head>
	<title>{t('reports.financialPosition.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('reports.financialPosition.title')}</Card.Title>
				<Card.Description>{t('reports.financialPosition.subtitle')}</Card.Description>
			</div>
		</Card.Header>

		<Card.Content class="flex flex-col gap-4">
			<div class="flex justify-start sm:justify-end">
				<Popover.Root bind:open={isAsOfOpen}>
					<Popover.Trigger
						class={cn(
							buttonVariants({ variant: 'outline' }),
							'h-9 w-full justify-start gap-2 px-3 text-left font-normal sm:w-70',
							!query.as_of && 'text-muted-foreground'
						)}
						disabled={isLoading}
						aria-label={t('reports.financialPosition.selectDate')}
					>
						<CalendarIcon class="size-4" />
						<span class="truncate">{getAsOfLabel()}</span>
					</Popover.Trigger>
					<Popover.Content align="end" class="w-auto p-0">
						<Calendar
							type="single"
							bind:value={asOfValue}
							class="rounded-md border-0 shadow-none"
						/>
						{#if query.as_of}
							<div class="flex justify-end border-t bg-popover p-2">
								<Button type="button" variant="ghost" size="sm" onclick={clearAsOfFilter}>
									<XIcon />
									{t('common.clear')}
								</Button>
							</div>
						{/if}
					</Popover.Content>
				</Popover.Root>
			</div>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('reports.financialPosition.errorLoad')} message={errorMessage} />
			{:else if isLoading || !position}
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
					{#each [...Array(4).keys()] as index (index)}
						<div class="space-y-2 rounded-md border p-4">
							<Skeleton class="h-4 w-24" />
							<Skeleton class="h-7 w-32" />
						</div>
					{/each}
				</div>
			{:else}
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">{t('reports.financialPosition.cash')}</p>
						<p class="text-2xl font-semibold">{formatIdr(position.cash)}</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">
							{t('reports.financialPosition.inventoryValue')}
						</p>
						<p class="text-2xl font-semibold">{formatIdr(position.inventory_value)}</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">
							{t('reports.financialPosition.totalAssets')}
						</p>
						<p class="text-2xl font-semibold">{formatIdr(position.total_assets)}</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">
							{t('reports.financialPosition.cumulativeNetProfit')}
						</p>
						<p class="text-2xl font-semibold">{formatIdr(position.cumulative_net_profit)}</p>
					</div>
				</div>

				<div class="rounded-md border">
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>{t('reports.financialPosition.colComponent')}</Table.Head>
								<Table.Head class="text-right"
									>{t('reports.financialPosition.colAmount')}</Table.Head
								>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							<Table.Row>
								<Table.Cell>{t('reports.financialPosition.asOf')}</Table.Cell>
								<Table.Cell class="text-right tabular-nums">
									{formatDate(position.as_of)}
								</Table.Cell>
							</Table.Row>
							{#each rows as row (row.label)}
								<Table.Row>
									<Table.Cell class={row.strong ? 'font-semibold' : undefined}>
										{row.label}
									</Table.Cell>
									<Table.Cell class={cn('text-right tabular-nums', row.strong && 'font-semibold')}>
										{formatIdr(row.value)}
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</div>
				<p class="text-sm text-muted-foreground">{t('reports.financialPosition.inventoryHint')}</p>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
