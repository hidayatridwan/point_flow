<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listEmployees,
		type Employee,
		type EmployeeListParams,
		type EmployeePagination
	} from '$lib/api/employees.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import EmployeeCreateDialog from './components/employee-create-dialog.svelte';
	import EmployeesTable from './components/employees-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: EmployeePagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let employees = $state<Employee[]>([]);
	let pagination = $state<EmployeePagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<EmployeeListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadEmployees(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadEmployees(params: EmployeeListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listEmployees(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadEmployees(query);
				return;
			}

			employees = result.employees;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			employees = [];
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function updatePageSize(pageSize: number) {
		if (!pageSizeOptions.includes(pageSize)) return;

		query = { ...query, limit: pageSize, page: 1 };
		syncQueryUrl(query);
		void loadEmployees(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadEmployees(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadEmployees(query);
		}, 350);
	}

	function updateEmployeeInList(updatedEmployee: Employee) {
		employees = employees.map((employee) =>
			employee.id === updatedEmployee.id ? updatedEmployee : employee
		);
	}

	async function refreshEmployeesAfterDelete() {
		await loadEmployees(query);
	}

	async function refreshEmployeesAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadEmployees(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<EmployeeListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search
		};
	}

	function syncQueryUrl(nextQuery: Required<EmployeeListParams>) {
		const nextUrl = new URL(page.url);

		if (nextQuery.page === defaultPagination.page) {
			nextUrl.searchParams.delete('page');
		} else {
			nextUrl.searchParams.set('page', String(nextQuery.page));
		}

		if (nextQuery.limit === defaultPagination.pageSize) {
			nextUrl.searchParams.delete('limit');
		} else {
			nextUrl.searchParams.set('limit', String(nextQuery.limit));
		}

		if (nextQuery.search) {
			nextUrl.searchParams.set('search', nextQuery.search);
		} else {
			nextUrl.searchParams.delete('search');
		}

		replaceState(resolve(`${nextUrl.pathname}${nextUrl.search}` as '/'), page.state);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.employees.errorLoad');
	}
</script>

<svelte:head>
	<title>Employees | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('master.employees.title')}</Card.Title>
				<Card.Description>{t('master.employees.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
					<PlusIcon />
					{t('master.employees.addBtn')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<SearchField
				value={searchValue}
				disabled={isLoading}
				ariaLabel={t('master.employees.searchAriaLabel')}
				placeholder={t('master.employees.searchPlaceholder')}
				onValueChange={updateSearchValue}
			/>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('master.employees.errorLoad')} message={errorMessage} />
			{:else}
				<EmployeesTable
					{employees}
					{isLoading}
					onEmployeeDeleted={refreshEmployeesAfterDelete}
					onEmployeeUpdated={updateEmployeeInList}
					emptyMessage={query.search
						? t('master.employees.emptySearch')
						: t('master.employees.emptyDefault')}
				/>
			{/if}
		</Card.Content>
		{#if !isLoading && !errorMessage}
			<Card.Footer>
				<DataPagination
					{pagination}
					{pageSizeOptions}
					onPageChange={updatePage}
					onPageSizeChange={updatePageSize}
				/>
			</Card.Footer>
		{/if}
	</Card.Root>

	<EmployeeCreateDialog bind:open={isCreateOpen} onCreated={refreshEmployeesAfterCreate} />
</div>
