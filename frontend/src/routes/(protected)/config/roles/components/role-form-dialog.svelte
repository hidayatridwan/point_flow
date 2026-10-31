<script lang="ts">
	import {
		createRole,
		replaceRolePermissions,
		updateRole,
		type Permission,
		type Role
	} from '$lib/api/roles.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	type RoleFormData = {
		code: string;
		name: string;
		description: string;
		permission_codes: string[];
	};

	let {
		open = $bindable(false),
		role = null,
		permissions,
		onSaved
	}: {
		open: boolean;
		role?: Role | null;
		permissions: Permission[];
		onSaved?: (role: Role) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let validationMessage = $state<string | null>(null);
	let formData = $state<RoleFormData>(getInitialFormData(null));

	const isEditing = $derived(Boolean(role));
	const dialogTitle = $derived(
		isEditing ? t('config.roles.editTitle') : t('config.roles.addTitle')
	);
	const submitLabel = $derived(isEditing ? t('config.roles.saveBtn') : t('config.roles.createBtn'));
	const submittingLabel = $derived(
		isEditing ? t('config.roles.saving') : t('config.roles.creating')
	);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		validationMessage = null;
		formData = getInitialFormData(role);
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const validationError = validateFormData(formData, role);
		validationMessage = validationError;
		if (validationError) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const payload = {
				code: formData.code.trim(),
				name: formData.name.trim(),
				description: formData.description.trim() || null
			};
			const savedRole = role
				? await saveExistingRole(role, payload)
				: await createRole({
						...payload,
						permission_codes: formData.permission_codes
					});

			await onSaved?.(savedRole);
			toast.success(isEditing ? t('config.roles.updatedToast') : t('config.roles.createdToast'), {
				description: t('config.roles.savedToastDesc').replace('{name}', savedRole.name)
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	async function saveExistingRole(
		role: Role,
		payload: { code: string; name: string; description: string | null }
	) {
		const updatedRole = await updateRole(role.id, {
			code: role.is_system ? undefined : payload.code,
			name: payload.name,
			description: payload.description
		});

		return replaceRolePermissions(updatedRole.id, {
			permission_codes: formData.permission_codes
		});
	}

	function togglePermission(code: string, checked: boolean) {
		formData = {
			...formData,
			permission_codes: checked
				? [...new Set([...formData.permission_codes, code])]
				: formData.permission_codes.filter((permissionCode) => permissionCode !== code)
		};
	}

	function hasPermission(code: string) {
		return formData.permission_codes.includes(code);
	}

	function getInitialFormData(role: Role | null | undefined): RoleFormData {
		return {
			code: role?.code ?? '',
			name: role?.name ?? '',
			description: role?.description ?? '',
			permission_codes: [...(role?.permissions ?? [])]
		};
	}

	function validateFormData(data: RoleFormData, role: Role | null | undefined) {
		const code = data.code.trim();
		const name = data.name.trim();

		if (!code) return t('common.validation.required').replace('{field}', t('config.roles.code'));
		if (!/^[a-z0-9_]+$/.test(code)) {
			return t('config.roles.invalidCodeChars');
		}
		if (!name) return t('common.validation.required').replace('{field}', t('common.name'));
		if (!role?.is_system && code.length > 80)
			return t('common.validation.maxChars')
				.replace('{field}', t('config.roles.code'))
				.replace('{min}', '80');
		if (name.length > 120)
			return t('common.validation.maxChars')
				.replace('{field}', t('common.name'))
				.replace('{min}', '120');
		return null;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return isEditing ? t('config.roles.errorSave') : t('config.roles.errorCreate');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-hidden p-0 sm:max-w-2xl">
		<form class="grid max-h-[90vh] grid-rows-[auto_1fr_auto]" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{dialogTitle}</Dialog.Title>
				<Dialog.Description>{t('config.roles.formDesc')}</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-5 overflow-y-auto px-6 py-5">
				<div class="grid gap-4 sm:grid-cols-2">
					<div class="grid gap-2">
						<Label for="role-code">{t('config.roles.code')}</Label>
						<Input
							id="role-code"
							placeholder={t('config.roles.codePlaceholder')}
							disabled={isSubmitting || role?.is_system}
							bind:value={formData.code}
						/>
					</div>

					<div class="grid gap-2">
						<Label for="role-name">{t('common.name')}</Label>
						<Input
							id="role-name"
							placeholder={t('config.roles.namePlaceholder')}
							disabled={isSubmitting}
							bind:value={formData.name}
						/>
					</div>
				</div>

				<div class="grid gap-2">
					<Label for="role-description">{t('common.description')}</Label>
					<Textarea
						id="role-description"
						placeholder={t('config.roles.descriptionPlaceholder')}
						disabled={isSubmitting}
						bind:value={formData.description}
					/>
				</div>

				<div class="grid gap-3">
					<div>
						<h3 class="text-sm font-medium">{t('config.roles.permissions')}</h3>
					</div>
					<div class="grid gap-2 rounded-md border p-3 sm:grid-cols-2">
						{#each permissions as permission (permission.code)}
							<label class="flex min-h-12 items-start gap-3 rounded-md px-2 py-2 hover:bg-muted/60">
								<input
									type="checkbox"
									class="mt-1 size-4 rounded border-input accent-primary"
									checked={hasPermission(permission.code)}
									disabled={isSubmitting}
									onchange={(event) =>
										togglePermission(permission.code, event.currentTarget.checked)}
								/>
								<span class="grid min-w-0 gap-0.5">
									<span class="truncate text-sm font-medium">{permission.name}</span>
									<span class="truncate font-mono text-xs text-muted-foreground">
										{permission.code}
									</span>
								</span>
							</label>
						{/each}
					</div>
				</div>

				{#if validationMessage}
					<ErrorAlert title={t('config.roles.invalidRole')} message={validationMessage} />
				{/if}

				{#if errorMessage}
					<ErrorAlert title={t('common.saveFailed')} message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('common.cancel')}
				</Dialog.Close>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? submittingLabel : submitLabel}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
