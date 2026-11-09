<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import UserCheckIcon from '@lucide/svelte/icons/user-check';
	import { listMembers, type Member } from '$lib/api/members.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { onDestroy } from 'svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		open = $bindable(false),
		onSelected
	}: {
		open: boolean;
		onSelected?: (member: Member) => void;
	} = $props();

	let members = $state<Member[]>([]);
	let search = $state('');
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	$effect(() => {
		if (!open) return;
		search = '';
		errorMessage = null;
		void loadMembers('');
	});

	function handleSearchInput(value: string) {
		search = value;
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(() => void loadMembers(value.trim()), 350);
	}

	async function loadMembers(searchValue: string) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listMembers({
				search: searchValue || undefined,
				is_active: true,
				limit: 50
			});
			if (requestId !== latestRequestId) return;
			members = result;
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

	function handleSelect(member: Member) {
		onSelected?.(member);
		open = false;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('point_flow.newSale.memberLoadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-md">
		<Dialog.Header class="px-6 pt-6">
			<Dialog.Title>{t('point_flow.newSale.memberDialogTitle')}</Dialog.Title>
			<Dialog.Description>{t('point_flow.newSale.memberDialogDesc')}</Dialog.Description>
		</Dialog.Header>

		<div class="grid gap-3 px-6 pb-6">
			<div class="relative">
				<SearchIcon class="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
				<Input
					class="pl-9"
					placeholder={t('point_flow.newSale.memberSearchPlaceholder')}
					aria-label={t('point_flow.newSale.memberSearchAriaLabel')}
					value={search}
					oninput={(event) => handleSearchInput(event.currentTarget.value)}
				/>
			</div>

			{#if errorMessage}
				<ErrorAlert title={t('point_flow.newSale.memberLoadFailed')} message={errorMessage} />
			{:else if isLoading}
				<div class="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
					<Spinner />
					{t('common.loading')}
				</div>
			{:else if members.length > 0}
				<div class="grid max-h-80 gap-2 overflow-y-auto">
					{#each members as member (member.id)}
						<div
							class="flex items-center justify-between gap-3 rounded-2xl border border-slate-100 bg-slate-50/70 p-3 dark:border-slate-800 dark:bg-slate-900/40"
						>
							<div class="min-w-0">
								<p class="truncate text-sm font-semibold">{member.full_name}</p>
								<p class="mt-0.5 truncate text-xs text-muted-foreground">
									{member.member_no} · {member.phone}
								</p>
							</div>
							<Button
								type="button"
								variant="outline"
								size="sm"
								class="h-9 shrink-0 rounded-xl"
								onclick={() => handleSelect(member)}
							>
								<UserCheckIcon />
								{t('common.add')}
							</Button>
						</div>
					{/each}
				</div>
			{:else}
				<p class="py-8 text-center text-sm text-muted-foreground">
					{t('point_flow.newSale.memberEmpty')}
				</p>
			{/if}
		</div>
	</Dialog.Content>
</Dialog.Root>
