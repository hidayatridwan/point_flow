<script lang="ts" module>
	export type PaginationState = {
		page: number;
		pageSize: number;
		totalRecords: number;
		totalPages?: number;
	};
</script>

<script lang="ts">
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ChevronsLeftIcon from '@lucide/svelte/icons/chevrons-left';
	import ChevronsRightIcon from '@lucide/svelte/icons/chevrons-right';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		pagination,
		pageSizeOptions,
		onPageChange,
		onPageSizeChange,
		disabled = false
	}: {
		pagination: PaginationState;
		pageSizeOptions: number[];
		onPageChange: (page: number) => void;
		onPageSizeChange: (pageSize: number) => void;
		disabled?: boolean;
	} = $props();

	const pageSize = $derived(Math.max(1, pagination.pageSize));
	const totalRecords = $derived(Math.max(0, pagination.totalRecords));
	const totalPages = $derived(
		Math.max(1, pagination.totalPages ?? Math.ceil(totalRecords / pageSize))
	);
	const currentPage = $derived(Math.min(Math.max(1, pagination.page), totalPages));
	const hasPreviousPage = $derived(currentPage > 1);
	const hasNextPage = $derived(currentPage < totalPages);
	const canGoToPreviousPage = $derived(!disabled && hasPreviousPage);
	const canGoToNextPage = $derived(!disabled && hasNextPage);
	const start = $derived(totalRecords === 0 ? 0 : (currentPage - 1) * pageSize + 1);
	const end = $derived(Math.min(totalRecords, currentPage * pageSize));
	const rowsPerPageItems = $derived(
		pageSizeOptions.map((option) => ({ value: String(option), label: String(option) }))
	);

	function handlePageSizeChange(value: string) {
		const pageSize = Number(value);
		if (!Number.isFinite(pageSize)) return;
		onPageSizeChange(pageSize);
	}
</script>

<div class="flex w-full flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
	<p class="text-sm text-muted-foreground">
		{t('common.resultsCount')
			.replace('{start}', String(start))
			.replace('{end}', String(end))
			.replace('{totalRecords}', String(totalRecords))}
	</p>

	<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-6 lg:gap-8">
		<div class="flex items-center gap-2">
			<p class="text-sm font-medium">{t('common.rowsPerPage')}</p>
			<Select.Root
				allowDeselect={false}
				type="single"
				value={String(pagination.pageSize)}
				items={rowsPerPageItems}
				{disabled}
				onValueChange={handlePageSizeChange}
			>
				<Select.Trigger class="h-8 w-18">{pagination.pageSize}</Select.Trigger>
				<Select.Content side="top">
					{#each pageSizeOptions as option (option)}
						<Select.Item value={String(option)} label={String(option)}>{option}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>

		<div class="flex w-28 items-center justify-start text-sm font-medium sm:justify-center">
			{t('common.pageOf')
				.replace('{page}', String(currentPage))
				.replace('{totalPages}', String(totalPages))}
		</div>

		<div class="flex items-center gap-2">
			<Button
				variant="outline"
				class="hidden size-8 p-0 lg:flex"
				disabled={!canGoToPreviousPage}
				onclick={() => onPageChange(1)}
				title={t('common.srGoFirstPage')}
			>
				<span class="sr-only">{t('common.srGoFirstPage')}</span>
				<ChevronsLeftIcon class="h-4 w-4" />
			</Button>
			<Button
				variant="outline"
				class="size-8 p-0"
				disabled={!canGoToPreviousPage}
				onclick={() => onPageChange(currentPage - 1)}
				title={t('common.srGoPrevPage')}
			>
				<span class="sr-only">{t('common.srGoPrevPage')}</span>
				<ChevronLeftIcon class="h-4 w-4" />
			</Button>
			<Button
				variant="outline"
				class="size-8 p-0"
				disabled={!canGoToNextPage}
				onclick={() => onPageChange(currentPage + 1)}
				title={t('common.srGoNextPage')}
			>
				<span class="sr-only">{t('common.srGoNextPage')}</span>
				<ChevronRightIcon class="h-4 w-4" />
			</Button>
			<Button
				variant="outline"
				class="hidden size-8 p-0 lg:flex"
				disabled={!canGoToNextPage}
				onclick={() => onPageChange(totalPages)}
				title={t('common.srGoLastPage')}
			>
				<span class="sr-only">{t('common.srGoLastPage')}</span>
				<ChevronsRightIcon class="h-4 w-4" />
			</Button>
		</div>
	</div>
</div>
