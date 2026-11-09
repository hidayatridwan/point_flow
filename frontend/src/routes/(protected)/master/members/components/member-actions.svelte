<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { Member } from '$lib/api/members.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import MemberDeleteAlertDialog from './member-delete-alert-dialog.svelte';
	import MemberDetailDialog from './member-detail-dialog.svelte';
	import MemberEditDialog from './member-edit-dialog.svelte';
	import MemberResetPasswordDialog from './member-reset-password-dialog.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		member,
		onDeleted,
		onUpdated
	}: {
		member: Member;
		onDeleted?: (memberId: number) => void | Promise<void>;
		onUpdated?: (member: Member) => void;
	} = $props();

	let isDetailOpen = $state(false);
	let isEditOpen = $state(false);
	let isDeleteOpen = $state(false);
	let isResetOpen = $state(false);
</script>

<div class="flex justify-center">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			{#snippet child({ props })}
				<Button
					{...props}
					variant="ghost"
					size="icon-sm"
					class="size-8"
					aria-label={`${t('common.action')}: ${member.full_name}`}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-36">
			<DropdownMenu.Item onSelect={() => (isDetailOpen = true)}>
				<EyeIcon />
				{t('common.detail')}
			</DropdownMenu.Item>
			<DropdownMenu.Item onSelect={() => (isEditOpen = true)}>
				<PencilIcon />
				{t('common.edit')}
			</DropdownMenu.Item>
			<DropdownMenu.Item onSelect={() => (isResetOpen = true)}>
				<KeyRoundIcon />
				{t('master.members.resetTitle')}
			</DropdownMenu.Item>
			<DropdownMenu.Item variant="destructive" onSelect={() => (isDeleteOpen = true)}>
				<TrashIcon />
				{t('common.delete')}
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<MemberDetailDialog memberId={member.id} bind:open={isDetailOpen} />
	<MemberEditDialog {member} bind:open={isEditOpen} {onUpdated} />
	<MemberResetPasswordDialog {member} bind:open={isResetOpen} />
	<MemberDeleteAlertDialog {member} bind:open={isDeleteOpen} {onDeleted} />
</div>
