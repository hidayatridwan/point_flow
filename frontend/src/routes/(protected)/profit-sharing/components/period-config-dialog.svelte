<script lang="ts">
	import { createPeriod, updatePeriod, type ProfitSharingPeriod } from '$lib/api/profit-sharing.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';

	const dateSchema = z
		.string()
		.trim()
		.regex(/^\d{4}-\d{2}-\d{2}$/, 'Use YYYY-MM-DD format.');

	const percentSchema = z.coerce
		.number({ error: 'Must be a number.' })
		.min(0, 'Must be between 0 and 100.')
		.max(100, 'Must be between 0 and 100.');

	const optionalPoolSchema = z.preprocess(
		(value) => (value === '' || value === null || value === undefined ? undefined : value),
		z.coerce.number({ error: 'Must be a number.' }).min(0, 'Must be zero or greater.').optional()
	);

	const periodSchema = z
		.object({
			period_name: z
				.string()
				.trim()
				.min(1, 'Period name is required.')
				.max(60, 'Period name must be at most 60 characters.'),
			start_date: dateSchema,
			end_date: dateSchema,
			total_pool: optionalPoolSchema,
			member_pct: percentSchema,
			reserve_pct: percentSchema,
			management_pct: percentSchema,
			supervisor_pct: percentSchema,
			social_pct: percentSchema,
			education_pct: percentSchema,
			notes: z.string().trim().max(2000, 'Notes must be at most 2000 characters.')
		})
		.superRefine((data, ctx) => {
			if (data.start_date > data.end_date) {
				ctx.addIssue({
					code: 'custom',
					path: ['end_date'],
					message: 'End date must not be before start date.'
				});
			}
			const total =
				data.member_pct +
				data.reserve_pct +
				data.management_pct +
				data.supervisor_pct +
				data.social_pct +
				data.education_pct;
			if (Math.abs(total - 100) > 0.01) {
				ctx.addIssue({
					code: 'custom',
					path: ['member_pct'],
					message: 'Allocation must total 100%.'
				});
			}
		});

	type PeriodForm = z.infer<typeof periodSchema>;

	const BUCKET_FIELDS = [
		{ field: 'member_pct', labelKey: 'finances.profitSharing.bucketMember' },
		{ field: 'reserve_pct', labelKey: 'finances.profitSharing.bucketReserve' },
		{ field: 'management_pct', labelKey: 'finances.profitSharing.bucketManagement' },
		{ field: 'supervisor_pct', labelKey: 'finances.profitSharing.bucketSupervisor' },
		{ field: 'social_pct', labelKey: 'finances.profitSharing.bucketSocial' },
		{ field: 'education_pct', labelKey: 'finances.profitSharing.bucketEducation' }
	] as const;

	let {
		open = $bindable(false),
		period = null,
		onSaved
	}: {
		open: boolean;
		period?: ProfitSharingPeriod | null;
		onSaved?: (id: number) => void | Promise<void>;
	} = $props();

	const isEditing = $derived(period !== null);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	function defaultFormData(source: ProfitSharingPeriod | null): PeriodForm {
		return {
			period_name: source?.period_name ?? '',
			start_date: source?.start_date ?? '',
			end_date: source?.end_date ?? '',
			total_pool: source?.total_pool ?? undefined,
			member_pct: source?.member_pct ?? 50,
			reserve_pct: source?.reserve_pct ?? 10,
			management_pct: source?.management_pct ?? 15,
			supervisor_pct: source?.supervisor_pct ?? 10,
			social_pct: source?.social_pct ?? 10,
			education_pct: source?.education_pct ?? 5,
			notes: source?.notes ?? ''
		};
	}

	const form = superForm(defaults(defaultFormData(null), zod4(periodSchema)), {
		id: 'profit-sharing-period',
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(periodSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		$formData = defaultFormData(period);
	});

	const allocationTotal = $derived(
		$formData.member_pct +
			$formData.reserve_pct +
			$formData.management_pct +
			$formData.supervisor_pct +
			$formData.social_pct +
			$formData.education_pct
	);

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const payload = {
				period_name: result.data.period_name,
				start_date: result.data.start_date,
				end_date: result.data.end_date,
				total_pool: result.data.total_pool ?? null,
				member_pct: result.data.member_pct,
				reserve_pct: result.data.reserve_pct,
				management_pct: result.data.management_pct,
				supervisor_pct: result.data.supervisor_pct,
				social_pct: result.data.social_pct,
				education_pct: result.data.education_pct,
				notes: emptyToNull(result.data.notes)
			};
			const saved = period ? await updatePeriod(period.id, payload) : await createPeriod(payload);
			await onSaved?.(saved.id);
			toast.success(
				isEditing
					? t('finances.profitSharing.updatedToast')
					: t('finances.profitSharing.createdToast')
			);
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return isEditing
			? t('finances.profitSharing.errorSave')
			: t('finances.profitSharing.errorCreate');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>
					{isEditing
						? t('finances.profitSharing.editTitle')
						: t('finances.profitSharing.configTitle')}
				</Dialog.Title>
				<Dialog.Description>
					{isEditing
						? t('finances.profitSharing.editDesc')
						: t('finances.profitSharing.configDesc')}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="period_name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>{t('finances.profitSharing.nameLabel')}</Form.Label>
							<Input
								{...props}
								placeholder={t('finances.profitSharing.namePlaceholder')}
								disabled={isSubmitting}
								bind:value={$formData.period_name}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<div class="grid gap-4 sm:grid-cols-2">
					<Form.Field {form} name="start_date">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{t('finances.profitSharing.startLabel')}</Form.Label>
								<Input
									{...props}
									placeholder="2026-01-01"
									disabled={isSubmitting}
									bind:value={$formData.start_date}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="end_date">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{t('finances.profitSharing.endLabel')}</Form.Label>
								<Input
									{...props}
									placeholder="2026-12-31"
									disabled={isSubmitting}
									bind:value={$formData.end_date}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
				</div>
				<p class="-mt-2 text-xs text-muted-foreground">{t('finances.profitSharing.dateHint')}</p>

				<Form.Field {form} name="total_pool">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>{t('finances.profitSharing.poolLabel')}</Form.Label>
							<Input
								{...props}
								type="number"
								min="0"
								step="0.01"
								placeholder={t('finances.profitSharing.poolPlaceholder')}
								disabled={isSubmitting}
								bind:value={$formData.total_pool}
							/>
						{/snippet}
					</Form.Control>
					<Form.Description>{t('finances.profitSharing.poolHint')}</Form.Description>
					<Form.FieldErrors />
				</Form.Field>

				<div class="grid gap-2">
					<p class="text-sm font-medium">{t('finances.profitSharing.allocTitle')}</p>
					<div class="grid grid-cols-2 gap-4 sm:grid-cols-3">
						{#each BUCKET_FIELDS as { field, labelKey } (field)}
							<Form.Field {form} name={field}>
								<Form.Control>
									{#snippet children({ props })}
										<Form.Label class="text-xs">{t(labelKey)}</Form.Label>
										<Input
											{...props}
											type="number"
											min="0"
											max="100"
											step="0.01"
											disabled={isSubmitting}
											bind:value={$formData[field]}
										/>
									{/snippet}
								</Form.Control>
								<Form.FieldErrors />
							</Form.Field>
						{/each}
					</div>
					<p
						class="text-xs {Math.abs(allocationTotal - 100) > 0.01
							? 'font-medium text-destructive'
							: 'text-muted-foreground'}"
					>
						{t('finances.profitSharing.allocTotal')}: {allocationTotal.toFixed(2)}%
						{#if Math.abs(allocationTotal - 100) > 0.01}
							— {t('finances.profitSharing.allocMustBe100')}
						{/if}
					</p>
				</div>

				<Form.Field {form} name="notes">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>{t('finances.profitSharing.notesLabel')}</Form.Label>
							<Textarea
								{...props}
								placeholder={t('finances.profitSharing.notesPlaceholder')}
								disabled={isSubmitting}
								bind:value={$formData.notes}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				{#if errorMessage}
					<ErrorAlert
						title={isEditing
							? t('finances.profitSharing.errorSave')
							: t('finances.profitSharing.errorCreate')}
						message={errorMessage}
					/>
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
					{isSubmitting
						? isEditing
							? t('finances.profitSharing.saving')
							: t('finances.profitSharing.creating')
						: isEditing
							? t('finances.profitSharing.saveBtn')
							: t('finances.profitSharing.createBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
