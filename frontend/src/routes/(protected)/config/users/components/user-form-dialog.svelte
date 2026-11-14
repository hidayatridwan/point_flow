<script lang="ts">
	import type { Role } from '$lib/api/roles.js';
	import { createUser, updateUser, type User } from '$lib/api/users.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	type UserFormData = {
		name: string;
		email: string;
		password: string;
		role_id: string;
		is_active: 'true' | 'false';
	};

	let {
		open = $bindable(false),
		user = null,
		roles,
		onSaved
	}: {
		open: boolean;
		user?: User | null;
		roles: Role[];
		onSaved?: (user: User) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let validationMessage = $state<string | null>(null);
	let formData = $state<UserFormData>(getInitialFormData(null, []));

	const isEditing = $derived(Boolean(user));
	const isAdminUser = $derived(user?.role_code === 'admin');
	const selectableRoles = $derived(getSelectableRoles(roles, user));

	// Label source untuk Select.Root: tanpa ini, trigger menampilkan ID mentah /
	// "true"/"false" sampai dropdown dibuka pertama kali (bits-ui lazy items).
	const roleItems = $derived(
		selectableRoles.map((role) => ({ value: String(role.id), label: role.name }))
	);
	const statusItems = $derived([
		{ value: 'true', label: t('status.active') },
		{ value: 'false', label: t('status.inactive') }
	]);
	const dialogTitle = $derived(
		isEditing ? t('config.users.editTitle') : t('config.users.addTitle')
	);
	const submitLabel = $derived(isEditing ? t('config.users.saveBtn') : t('config.users.createBtn'));
	const submittingLabel = $derived(
		isEditing ? t('config.users.saving') : t('config.users.creating')
	);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		validationMessage = null;
		formData = getInitialFormData(user, roles);
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const validationError = validateFormData(formData, user, selectableRoles);
		validationMessage = validationError;
		if (validationError) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const roleId = Number(formData.role_id);
			const savedUser = user
				? await updateUser(user.id, {
						name: formData.name.trim(),
						email: formData.email.trim(),
						password: formData.password.trim() || undefined,
						role_id: isAdminUser ? undefined : roleId,
						is_active: formData.is_active === 'true'
					})
				: await createUser({
						name: formData.name.trim(),
						email: formData.email.trim(),
						password: formData.password.trim(),
						role_id: roleId,
						is_active: formData.is_active === 'true'
					});

			await onSaved?.(savedUser);
			toast.success(isEditing ? t('config.users.updatedToast') : t('config.users.createdToast'), {
				description: t('config.users.savedToastDesc').replace('{name}', savedUser.name)
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getInitialFormData(user: User | null | undefined, roles: Role[]): UserFormData {
		const nonAdminRole = roles.find((role) => role.code !== 'admin');

		return {
			name: user?.name ?? '',
			email: user?.email ?? '',
			password: '',
			role_id: String(user?.role_id ?? nonAdminRole?.id ?? ''),
			is_active: String(user?.is_active ?? true) as UserFormData['is_active']
		};
	}

	function getSelectableRoles(roles: Role[], user: User | null | undefined) {
		if (user?.role_code === 'admin') {
			return roles.filter((role) => role.code === 'admin');
		}

		return roles.filter((role) => role.code !== 'admin');
	}

	function validateFormData(
		data: UserFormData,
		user: User | null | undefined,
		selectableRoles: Role[]
	) {
		const name = data.name.trim();
		const email = data.email.trim();
		const password = data.password.trim();
		const roleId = Number(data.role_id);
		const selectedRole = selectableRoles.find((role) => role.id === roleId);

		if (!name) return t('common.validation.required').replace('{field}', t('common.name'));
		if (name.length > 120)
			return t('common.validation.maxChars')
				.replace('{field}', t('common.name'))
				.replace('{min}', '120');
		if (!email) return t('common.validation.required').replace('{field}', t('common.email'));
		if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) return t('common.validation.emailInvalid');
		if (!user && !password)
			return t('common.validation.required').replace('{field}', t('login.password'));
		if (password && password.length < 8)
			return t('common.validation.minChars')
				.replace('{field}', t('login.password'))
				.replace('{min}', '8');
		if (!selectedRole)
			return t('common.validation.required').replace('{field}', t('config.users.role'));
		if (!user && selectedRole.code === 'admin') return t('config.users.adminRoleForbidden');
		if (user?.role_code !== 'admin' && selectedRole.code === 'admin') {
			return t('config.users.adminRoleForbidden');
		}
		return null;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return isEditing ? t('config.users.errorSave') : t('config.users.errorCreate');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-2xl">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{dialogTitle}</Dialog.Title>
				<Dialog.Description>{t('config.users.formDesc')}</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-5 px-6">
				<div class="grid gap-4 sm:grid-cols-2">
					<div class="grid gap-2">
						<Label for="user-name">{t('common.name')}</Label>
						<Input
							id="user-name"
							placeholder="Cashier"
							disabled={isSubmitting}
							bind:value={formData.name}
						/>
					</div>

					<div class="grid gap-2">
						<Label for="user-email">{t('common.email')}</Label>
						<Input
							id="user-email"
							type="email"
							placeholder="cashier@example.com"
							disabled={isSubmitting}
							bind:value={formData.email}
						/>
					</div>
				</div>

				<div class="grid gap-4 sm:grid-cols-2">
					<div class="grid gap-2">
						<Label for="user-password"
							>{isEditing ? t('config.users.newPassword') : t('login.password')}</Label
						>
						<Input
							id="user-password"
							type="password"
							placeholder={isEditing
								? t('config.users.passwordEditPlaceholder')
								: t('config.users.passwordPlaceholder')}
							disabled={isSubmitting}
							bind:value={formData.password}
						/>
					</div>

					<div class="grid gap-2">
						<Label for="user-role">{t('config.users.role')}</Label>
						<Select.Root
							allowDeselect={false}
							type="single"
							items={roleItems}
							value={formData.role_id}
							disabled={isSubmitting || isAdminUser}
							onValueChange={(value) => {
								formData.role_id = value;
							}}
						>
							<Select.Trigger id="user-role" class="w-full">
								<Select.Value placeholder={t('config.users.selectRole')} />
							</Select.Trigger>
							<Select.Content>
								{#each selectableRoles as role (role.id)}
									<Select.Item value={String(role.id)} label={role.name}>{role.name}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>
				</div>

				<div class="grid gap-2 sm:max-w-xs">
					<Label for="user-status">{t('common.status')}</Label>
					<Select.Root
						allowDeselect={false}
						type="single"
						items={statusItems}
						value={formData.is_active}
						disabled={isSubmitting}
						onValueChange={(value) => {
							if (value === 'true' || value === 'false') {
								formData.is_active = value;
							}
						}}
					>
						<Select.Trigger id="user-status" class="w-full">
							<Select.Value placeholder={t('common.selectStatus')} />
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="true" label={t('status.active')}>{t('status.active')}</Select.Item
							>
							<Select.Item value="false" label={t('status.inactive')}
								>{t('status.inactive')}</Select.Item
							>
						</Select.Content>
					</Select.Root>
				</div>

				{#if validationMessage}
					<ErrorAlert title={t('config.users.invalidUser')} message={validationMessage} />
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
				<Button type="submit" disabled={isSubmitting || selectableRoles.length === 0}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? submittingLabel : submitLabel}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
