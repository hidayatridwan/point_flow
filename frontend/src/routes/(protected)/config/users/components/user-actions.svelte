<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import type { Role } from '$lib/api/roles.js';
	import type { User } from '$lib/api/users.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import UserDeleteAlertDialog from './user-delete-alert-dialog.svelte';
	import UserDetailDialog from './user-detail-dialog.svelte';
	import UserFormDialog from './user-form-dialog.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		user,
		roles,
		onDeleted,
		onUpdated
	}: {
		user: User;
		roles: Role[];
		onDeleted?: (userId: number) => void | Promise<void>;
		onUpdated?: (user: User) => void;
	} = $props();

	let isDetailOpen = $state(false);
	let isEditOpen = $state(false);
	let isDeleteOpen = $state(false);
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
					aria-label={`${t('common.action')}: ${user.name}`}
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
			<DropdownMenu.Item variant="destructive" onSelect={() => (isDeleteOpen = true)}>
				<TrashIcon />
				{t('common.delete')}
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<UserDetailDialog userId={user.id} bind:open={isDetailOpen} />
	<UserFormDialog {user} {roles} bind:open={isEditOpen} onSaved={onUpdated} />
	<UserDeleteAlertDialog {user} bind:open={isDeleteOpen} {onDeleted} />
</div>
