<script lang="ts">
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import {
		cashDrawerMovementTypes,
		createCashDrawerMovement,
		type CashDrawerMovement,
		type CashDrawerMovementType,
		type CashRegisterSession
	} from '$lib/api/cash-registers.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	type MovementTypeOption = {
		value: CashDrawerMovementType;
		label: string;
		description: string;
		direction: 'in' | 'out';
		reasonPlaceholder: string;
	};

	let {
		activeSession,
		open = $bindable(false),
		onCreated
	}: {
		activeSession: CashRegisterSession | null;
		open: boolean;
		onCreated?: (movement: CashDrawerMovement) => void | Promise<void>;
	} = $props();

	const movementTypeOptions: MovementTypeOption[] = [
		{
			value: 'cash_in',
			label: 'Cash In',
			description: 'Adds manual cash into expected drawer cash.',
			direction: 'in',
			reasonPlaceholder: 'Additional drawer cash'
		},
		{
			value: 'cash_drop',
			label: 'Cash Drop',
			description: 'Removes cash from drawer to safe/back office.',
			direction: 'out',
			reasonPlaceholder: 'Midday cash drop'
		},
		{
			value: 'cash_out',
			label: 'Cash Out',
			description: 'Removes cash from drawer outside sale/refund transactions.',
			direction: 'out',
			reasonPlaceholder: 'Cash removed from drawer'
		},
		{
			value: 'petty_cash',
			label: 'Petty Cash',
			description: 'Records small operational expense paid from drawer.',
			direction: 'out',
			reasonPlaceholder: 'Small operational expense'
		}
	];

	let movementType = $state<CashDrawerMovementType>('cash_in');
	let amount = $state('');
	let reason = $state('');
	let notes = $state('');
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const selectedTypeOption = $derived(
		movementTypeOptions.find((option) => option.value === movementType) ?? movementTypeOptions[0]
	);
	const amountValue = $derived(toNumber(amount));
	const directionLabel = $derived(selectedTypeOption.direction === 'in' ? 'Cash In' : 'Cash Out');

	$effect(() => {
		if (!open) return;
		movementType = 'cash_in';
		amount = '';
		reason = '';
		notes = '';
		errorMessage = null;
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		const validationError = getValidationError();
		if (validationError) {
			errorMessage = validationError;
			return;
		}

		isSubmitting = true;
		errorMessage = null;

		try {
			const movement = await createCashDrawerMovement({
				cash_register_session_id: activeSession?.id as number,
				movement_type: movementType,
				amount: amountValue,
				reason: emptyToNull(reason),
				notes: emptyToNull(notes)
			});
			await onCreated?.(movement);
			toast.success(t('pos.cashMovements.createdToast'), {
				description: `${movement.movement_no} ${formatIdr(movement.amount)}.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function handleMovementTypeChange(value: string) {
		if (!cashDrawerMovementTypes.includes(value as CashDrawerMovementType)) return;
		movementType = value as CashDrawerMovementType;
	}

	function getValidationError() {
		if (!activeSession) return t('pos.cashMovements.noActiveRegisterAlertDesc');
		if (!Number.isFinite(amountValue) || amountValue <= 0) {
			return t('common.validation.positiveNumber').replace(
				'{field}',
				t('pos.cashMovements.amount')
			);
		}
		return null;
	}

	function toNumber(value: string) {
		const number = Number(value);
		return Number.isFinite(number) ? number : 0;
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('pos.cashMovements.errorCreate');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-xl">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('pos.cashMovements.addTitle')}</Dialog.Title>
				<Dialog.Description>
					{t('pos.cashMovements.addDesc')}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				{#if activeSession}
					<div class="grid gap-1 rounded-md border bg-muted/20 p-3 text-sm">
						<div class="font-medium">
							{activeSession.cash_register_code} - {activeSession.session_no}
						</div>
						<div class="text-muted-foreground">
							Opening cash {formatIdr(activeSession.opening_cash)}
						</div>
					</div>
				{:else}
					<ErrorAlert
						title={t('pos.cashMovements.noActiveRegisterAlert')}
						message={t('pos.cashMovements.noActiveRegisterAlertDesc')}
					/>
				{/if}

				<div class="grid gap-2">
					<Label>{t('pos.cashMovements.movementType')}</Label>
					<Select.Root
						allowDeselect={false}
						type="single"
						value={movementType}
						disabled={isSubmitting}
						onValueChange={handleMovementTypeChange}
					>
						<Select.Trigger class="w-full">
							<Select.Value placeholder={t('pos.cashMovements.selectMovementType')} />
						</Select.Trigger>
						<Select.Content>
							{#each movementTypeOptions as option (option.value)}
								<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
					<p class="text-sm text-muted-foreground">{selectedTypeOption.description}</p>
				</div>

				<div class="grid gap-2">
					<Label for="cash-movement-amount">{t('pos.cashMovements.amount')}</Label>
					<Input
						id="cash-movement-amount"
						type="number"
						min="0"
						step="1"
						placeholder="0"
						disabled={isSubmitting}
						bind:value={amount}
					/>
				</div>

				<div class="grid gap-2 rounded-md bg-muted/30 p-3 text-sm">
					<div class="flex justify-between gap-3">
						<span class="text-muted-foreground">{t('pos.cashMovements.closingImpact')}</span>
						<span class="font-medium">{directionLabel}</span>
					</div>
					<div class="flex justify-between gap-3">
						<span class="text-muted-foreground">{t('pos.cashMovements.amount')}</span>
						<span class="font-medium">{formatIdr(amountValue)}</span>
					</div>
				</div>

				<div class="grid gap-2">
					<Label for="cash-movement-reason">{t('pos.cashMovements.reason')}</Label>
					<Input
						id="cash-movement-reason"
						placeholder={selectedTypeOption.reasonPlaceholder}
						disabled={isSubmitting}
						bind:value={reason}
					/>
				</div>

				<div class="grid gap-2">
					<Label for="cash-movement-notes">{t('pos.cashMovements.notes')}</Label>
					<Textarea
						id="cash-movement-notes"
						placeholder={t('pos.cashMovements.notesPlaceholder')}
						disabled={isSubmitting}
						bind:value={notes}
					/>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('pos.cashMovements.errorCreate')} message={errorMessage} />
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
				<Button type="submit" disabled={isSubmitting || !activeSession}>
					{#if isSubmitting}
						<Spinner />
					{:else}
						<BanknoteIcon />
					{/if}
					{isSubmitting ? t('pos.cashMovements.creating') : t('pos.cashMovements.createBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
