<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { listRoles, type Role } from '$lib/api/roles.js';
	import {
		listUsers,
		type User,
		type UserListParams,
		type UserPagination
	} from '$lib/api/users.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import UserFormDialog from './components/user-form-dialog.svelte';
	import UsersTable from './components/users-table.svelte';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: UserPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let users = $state<User[]>([]);
	let roles = $state<Role[]>([]);
	let pagination = $state<UserPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<UserListParams>>(initialQuery);
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

	async function loadPageData(params: UserListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const [userResult, roleResult] = await Promise.all([
				listUsers(params),
				listRoles({ limit: 100 })
			]);
			if (requestId !== latestRequestId) return;

			if (
				userResult.pagination.totalPages > 0 &&
				userResult.pagination.page > userResult.pagination.totalPages
			) {
				query = { ...query, page: userResult.pagination.totalPages };
				syncQueryUrl(query);
				await loadPageData(query);
				return;
			}

			users = userResult.users;
			roles = roleResult.roles;
			pagination = userResult.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			users = [];
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

	function updateUserInList(updatedUser: User) {
		users = users.map((user) => (user.id === updatedUser.id ? updatedUser : user));
	}

	async function refreshUsersAfterDelete() {
		await loadPageData(query);
	}

	async function refreshUsersAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadPageData(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<UserListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search
		};
	}

	function syncQueryUrl(nextQuery: Required<UserListParams>) {
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
		return 'Failed to load users.';
	}
</script>

<svelte:head>
	<title>Users | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader title="Users" description="Manage user sign-in access, status, and role assignment.">
		{#snippet actions()}
			<Button class="w-full sm:w-auto" onclick={openCreateDialog} disabled={isLoading}>
				<PlusIcon />
				Add User
			</Button>
		{/snippet}
	</PageHeader>

	<section class="flex flex-col gap-3" aria-labelledby="users-table-title">
		<h2 id="users-table-title" class="sr-only">User records</h2>

		<SearchField
			value={searchValue}
			disabled={isLoading}
			ariaLabel="Search users"
			placeholder="Search users..."
			onValueChange={updateSearchValue}
		/>

		{#if errorMessage && !isLoading}
			<ErrorAlert title="Failed to load users." message={errorMessage} />
		{:else}
			<UsersTable
				{users}
				{roles}
				{isLoading}
				onUserDeleted={refreshUsersAfterDelete}
				onUserUpdated={updateUserInList}
				emptyMessage={query.search ? 'No users match your search.' : 'No users yet.'}
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

	<UserFormDialog bind:open={isCreateOpen} {roles} onSaved={refreshUsersAfterCreate} />
</div>
