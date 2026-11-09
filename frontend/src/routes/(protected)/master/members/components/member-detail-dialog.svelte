<script lang="ts">
	import { getMember, type Member } from '$lib/api/members.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		memberId,
		open = $bindable(false)
	}: {
		memberId: number;
		open: boolean;
	} = $props();

	let member = $state<Member | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadMemberDetail(memberId);
	});

	async function loadMemberDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getMember(id);
			if (requestId !== latestRequestId) return;
			member = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			member = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.members.errorLoadDetail');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>{t('master.members.detailTitle')}</Dialog.Title>
			<Dialog.Description>{t('master.members.detailDesc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if member}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('master.members.memberNo')} value={member.member_no} />
					<DetailField label={t('common.status')}>
						<StatusBadge active={member.is_active} />
					</DetailField>
					<DetailField label={t('common.name')} value={member.full_name} />
					<DetailField label={t('common.phone')} value={member.phone} />
					<DetailField label={t('common.email')} value={member.email || '-'} />
					<DetailField
						label={t('master.members.joinedAt')}
						value={formatDateTime(member.joined_at)}
					/>
				</div>
				<div class="grid grid-cols-1 gap-3">
					<DetailField label={t('common.address')} value={member.address || '-'} />
					<DetailField label={t('common.notes')} value={member.notes || '-'} />
				</div>
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('common.createdAt')} value={formatDateTime(member.created_at)} />
					<DetailField label={t('common.updatedAt')} value={formatDateTime(member.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
