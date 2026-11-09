<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteMember, type Member } from '$lib/api/members.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		member,
		open = $bindable(false),
		onDeleted
	}: {
		member: Member;
		open: boolean;
		onDeleted?: (memberId: number) => void | Promise<void>;
	} = $props();

	let isDeleting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handleDelete(event: MouseEvent) {
		event.preventDefault();
		if (isDeleting) return;

		isDeleting = true;
		errorMessage = null;

		try {
			const result = await deleteMember(member.id);
			await onDeleted?.(result.id);
			if (result.is_active) {
				toast.success(t('master.members.deletedToast'), {
					description: `${result.full_name} ${t('common.wasRemoved')}`
				});
			} else {
				toast.success(t('master.members.deactivatedToast'), {
					description: result.full_name
				});
			}
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isDeleting = false;
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.members.errorDelete');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('master.members.deleteTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				<span class="font-medium text-foreground">{member.full_name}</span> — {t(
					'master.members.deleteDesc'
				).replace('{name}', '')}
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title={t('common.deleteFailed')} message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isDeleting}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action variant="destructive" disabled={isDeleting} onclick={handleDelete}>
				<TrashIcon />
				{isDeleting ? t('common.deleting') : t('common.delete')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
