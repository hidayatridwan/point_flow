<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listPermissions,
		listRoles,
		type Permission,
		type Role,
		type RoleListParams,
		type RolePagination
	} from '$lib/api/roles.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import RoleFormDialog from './components/role-form-dialog.svelte';
	import RolesTable from './components/roles-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: RolePagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let roles = $state<Role[]>([]);
	let permissions = $state<Permission[]>([]);
	let pagination = $state<RolePagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<RoleListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadPageData(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadPageData(params: RoleListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const [roleResult, permissionResult] = await Promise.all([
				listRoles(params),
				listPermissions({ limit: 100 })
			]);
			if (requestId !== latestRequestId) return;

			if (
				roleResult.pagination.totalPages > 0 &&
				roleResult.pagination.page > roleResult.pagination.totalPages
			) {
				query = { ...query, page: roleResult.pagination.totalPages };
				syncQueryUrl(query);
				await loadPageData(query);
				return;
			}

			roles = roleResult.roles;
			permissions = permissionResult;
			pagination = roleResult.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			roles = [];
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
		void loadPageData(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadPageData(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadPageData(query);
		}, 350);
	}

	function updateRoleInList(updatedRole: Role) {
		roles = roles.map((role) => (role.id === updatedRole.id ? updatedRole : role));
	}

	async function refreshRolesAfterDelete() {
		await loadPageData(query);
	}

	async function refreshRolesAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadPageData(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<RoleListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search
		};
	}

	function syncQueryUrl(nextQuery: Required<RoleListParams>) {
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
		return t('config.roles.errorLoad');
	}
</script>

<svelte:head>
	<title>Roles | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader title={t('config.roles.title')} description={t('config.roles.subtitle')}>
		{#snippet actions()}
			<Button class="w-full sm:w-auto" onclick={openCreateDialog} disabled={isLoading}>
				<PlusIcon />
				{t('config.roles.addBtn')}
			</Button>
		{/snippet}
	</PageHeader>

	<section class="flex flex-col gap-3" aria-labelledby="roles-table-title">
		<h2 id="roles-table-title" class="sr-only">Role records</h2>

		<SearchField
			value={searchValue}
			disabled={isLoading}
			ariaLabel={t('config.roles.searchAriaLabel')}
			placeholder={t('config.roles.searchPlaceholder')}
			onValueChange={updateSearchValue}
		/>

		{#if errorMessage && !isLoading}
			<ErrorAlert title={t('config.roles.errorLoad')} message={errorMessage} />
		{:else}
			<RolesTable
				{roles}
				{permissions}
				{isLoading}
				onRoleDeleted={refreshRolesAfterDelete}
				onRoleUpdated={updateRoleInList}
				emptyMessage={query.search ? t('config.roles.emptySearch') : t('config.roles.emptyDefault')}
			/>
			{#if !isLoading}
				<DataPagination
					{pagination}
					{pageSizeOptions}
					onPageChange={updatePage}
					onPageSizeChange={updatePageSize}
				/>
			{/if}
		{/if}
	</section>

	<RoleFormDialog bind:open={isCreateOpen} {permissions} onSaved={refreshRolesAfterCreate} />
</div>
