<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		getIncomeStatement,
		type FinancialPeriodParams,
		type IncomeStatement
	} from '$lib/api/reports.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDate, formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let statement = $state<IncomeStatement | null>(null);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<FinancialPeriodParams>>(initialQuery);
	let dateRangeValue = $state<DateRange>({
		start: dateStringToDateValue(initialQuery.date_from),
		end: dateStringToDateValue(initialQuery.date_to)
	});
	let isDateRangeOpen = $state(false);
	let latestRequestId = 0;

	const rows = $derived(
		statement
			? [
					{
						label: t('reports.incomeStatement.grossRevenue'),
						value: statement.gross_revenue,
						strong: false
					},
					{
						label: t('reports.incomeStatement.returnAmount'),
						value: statement.return_amount,
						strong: false
					},
					{
						label: t('reports.incomeStatement.netRevenue'),
						value: statement.net_revenue,
						strong: true
					},
					{ label: t('reports.incomeStatement.cogs'), value: statement.cogs, strong: false },
					{
						label: t('reports.incomeStatement.grossProfit'),
						value: statement.gross_profit,
						strong: true
					},
					{
						label: t('reports.incomeStatement.payrollExpense'),
						value: statement.payroll_expense,
						strong: false
					},
					{
						label: t('reports.incomeStatement.otherExpense'),
						value: statement.other_expense,
						strong: false
					},
					{
						label: t('reports.incomeStatement.netProfit'),
						value: statement.net_profit,
						strong: true
					}
				]
			: []
	);

	onMount(() => {
		void loadStatement(query);
	});

	$effect(() => {
		const date_from = dateValueToDateString(dateRangeValue.start);
		const date_to = dateValueToDateString(dateRangeValue.end);
		if (date_from === query.date_from && date_to === query.date_to) return;

		updateDateRangeFilter(date_from, date_to);
	});

	async function loadStatement(params: FinancialPeriodParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getIncomeStatement(params);
			if (requestId !== latestRequestId) return;
			statement = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			statement = null;
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function updateDateRangeFilter(date_from: string, date_to: string) {
		query = { date_from, date_to };
		syncQueryUrl(query);
		void loadStatement(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	function parseQuery(searchParams: URLSearchParams): Required<FinancialPeriodParams> {
		return {
			date_from: searchParams.get('date_from') ?? '',
			date_to: searchParams.get('date_to') ?? ''
		};
	}

	function syncQueryUrl(nextQuery: Required<FinancialPeriodParams>) {
		const nextUrl = new URL(page.url);
		setOptionalSearchParam(nextUrl, 'date_from', nextQuery.date_from);
		setOptionalSearchParam(nextUrl, 'date_to', nextQuery.date_to);
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

	function dateValueToDateString(value: DateValue | undefined) {
		return value?.toString() ?? '';
	}

	function getDateRangeLabel() {
		if (query.date_from && query.date_to)
			return `${formatDate(query.date_from)} - ${formatDate(query.date_to)}`;
		if (query.date_from) return `${t('common.from')} ${formatDate(query.date_from)}`;
		if (query.date_to) return `${t('common.until')} ${formatDate(query.date_to)}`;
		return t('common.selectDateRange');
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('reports.incomeStatement.errorLoad');
	}
</script>

<svelte:head>
	<title>{t('reports.incomeStatement.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('reports.incomeStatement.title')}</Card.Title>
				<Card.Description>{t('reports.incomeStatement.subtitle')}</Card.Description>
			</div>
		</Card.Header>

		<Card.Content class="flex flex-col gap-4">
			<div class="flex justify-start sm:justify-end">
				<Popover.Root bind:open={isDateRangeOpen}>
					<Popover.Trigger
						class={cn(
							buttonVariants({ variant: 'outline' }),
							'h-9 w-full justify-start gap-2 px-3 text-left font-normal sm:w-70',
							!query.date_from && !query.date_to && 'text-muted-foreground'
						)}
						disabled={isLoading}
						aria-label={t('common.filterByDateRange')}
					>
						<CalendarIcon class="size-4" />
						<span class="truncate">{getDateRangeLabel()}</span>
					</Popover.Trigger>
					<Popover.Content align="end" class="w-auto max-w-[calc(100vw-2rem)] overflow-x-auto p-0">
						<RangeCalendar
							bind:value={dateRangeValue}
							class="rounded-md border-0 shadow-none"
							numberOfMonths={2}
						/>
						{#if query.date_from || query.date_to}
							<div class="flex justify-end border-t bg-popover p-2">
								<Button type="button" variant="ghost" size="sm" onclick={clearDateRangeFilter}>
									<XIcon />
									{t('common.clear')}
								</Button>
							</div>
						{/if}
					</Popover.Content>
				</Popover.Root>
			</div>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('reports.incomeStatement.errorLoad')} message={errorMessage} />
			{:else if isLoading || !statement}
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
						<p class="text-sm text-muted-foreground">{t('reports.incomeStatement.netRevenue')}</p>
						<p class="text-2xl font-semibold">{formatIdr(statement.net_revenue)}</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">{t('reports.incomeStatement.grossProfit')}</p>
						<p class="text-2xl font-semibold">{formatIdr(statement.gross_profit)}</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">{t('reports.incomeStatement.cogs')}</p>
						<p class="text-2xl font-semibold">{formatIdr(statement.cogs)}</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">{t('reports.incomeStatement.netProfit')}</p>
						<p class="text-2xl font-semibold">{formatIdr(statement.net_profit)}</p>
					</div>
				</div>

				<div class="rounded-md border">
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>{t('reports.incomeStatement.colComponent')}</Table.Head>
								<Table.Head class="text-right">{t('reports.incomeStatement.colAmount')}</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
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
			{/if}
		</Card.Content>
	</Card.Root>
</div>
