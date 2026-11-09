<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { listMembers, type Member } from '$lib/api/members.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import MemberCreateDialog from './components/member-create-dialog.svelte';
	import MembersTable from './components/members-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	type StatusFilter = 'all' | 'active' | 'inactive';

	let members = $state<Member[]>([]);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<{ search: string; status: StatusFilter }>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadMembers(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadMembers(params: { search: string; status: StatusFilter }) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			members = await listMembers({
				search: params.search || undefined,
				is_active: params.status === 'all' ? undefined : params.status === 'active'
			});
			if (requestId !== latestRequestId) return;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			members = [];
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim() };
			syncQueryUrl(query);
			void loadMembers(query);
		}, 350);
	}

	function updateStatusFilter(status: StatusFilter) {
		if (status === query.status) return;
		query = { ...query, status };
		syncQueryUrl(query);
		void loadMembers(query);
	}

	function updateMemberInList(updatedMember: Member) {
		members = members.map((member) => (member.id === updatedMember.id ? updatedMember : member));
	}

	async function refreshMembersAfterChange() {
		await loadMembers(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): { search: string; status: StatusFilter } {
		const search = searchParams.get('search') ?? '';
		const statusParam = searchParams.get('status');
		const status: StatusFilter =
			statusParam === 'active' || statusParam === 'inactive' ? statusParam : 'all';
		return { search, status };
	}

	function syncQueryUrl(nextQuery: { search: string; status: StatusFilter }) {
		const nextUrl = new URL(page.url);

		if (nextQuery.search) {
			nextUrl.searchParams.set('search', nextQuery.search);
		} else {
			nextUrl.searchParams.delete('search');
		}

		if (nextQuery.status === 'all') {
			nextUrl.searchParams.delete('status');
		} else {
			nextUrl.searchParams.set('status', nextQuery.status);
		}

		replaceState(resolve(`${nextUrl.pathname}${nextUrl.search}` as '/'), page.state);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.members.errorLoad');
	}
</script>

<svelte:head>
	<title>{t('master.members.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('master.members.title')}</Card.Title>
				<Card.Description>{t('master.members.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
					<PlusIcon />
					{t('master.members.addBtn')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<div class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_220px]">
				<SearchField
					value={searchValue}
					disabled={isLoading}
					ariaLabel={t('master.members.searchAriaLabel')}
					placeholder={t('master.members.searchPlaceholder')}
					onValueChange={updateSearchValue}
				/>
				<Select.Root
					type="single"
					value={query.status}
					disabled={isLoading}
					onValueChange={(value) => {
						if (value === 'active' || value === 'inactive' || value === 'all') {
							updateStatusFilter(value);
						}
					}}
				>
					<Select.Trigger class="w-full" aria-label={t('master.members.statusFilter')}>
						<Select.Value placeholder={t('master.members.statusAll')} />
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="all" label={t('master.members.statusAll')}
							>{t('master.members.statusAll')}</Select.Item
						>
						<Select.Item value="active" label={t('status.active')}>{t('status.active')}</Select.Item
						>
						<Select.Item value="inactive" label={t('status.inactive')}
							>{t('status.inactive')}</Select.Item
						>
					</Select.Content>
				</Select.Root>
			</div>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('master.members.errorLoad')} message={errorMessage} />
			{:else}
				<MembersTable
					{members}
					{isLoading}
					onMemberDeleted={refreshMembersAfterChange}
					onMemberUpdated={updateMemberInList}
					emptyMessage={query.search || query.status !== 'all'
						? t('master.members.emptySearch')
						: t('master.members.emptyDefault')}
				/>
			{/if}
		</Card.Content>
	</Card.Root>

	<MemberCreateDialog bind:open={isCreateOpen} onCreated={refreshMembersAfterChange} />
</div>
