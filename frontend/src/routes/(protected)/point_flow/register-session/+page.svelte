<script lang="ts">
	import CircleDollarSignIcon from '@lucide/svelte/icons/circle-dollar-sign';
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import LockIcon from '@lucide/svelte/icons/lock';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import UnlockIcon from '@lucide/svelte/icons/unlock';
	import { onMount } from 'svelte';
	import {
		clearStoredActiveCashRegisterSessionId,
		closeCashRegisterSession,
		getStoredActiveCashRegisterSessionId,
		listCashRegisterSessions,
		listCashRegisters,
		openCashRegisterSession,
		setStoredActiveCashRegisterSessionId,
		type CashRegister,
		type CashRegisterSession
	} from '$lib/api/cash-registers.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { formatDateTime, formatIdr } from '$lib/formatters.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let registers = $state<CashRegister[]>([]);
	let openSessions = $state<CashRegisterSession[]>([]);
	let activeSession = $state<CashRegisterSession | null>(null);
	let closedSession = $state<CashRegisterSession | null>(null);
	let selectedCashRegisterId = $state('');
	let openingCash = $state('');
	let openingNotes = $state('');
	let closingCashCounted = $state('');
	let closingNotes = $state('');
	let isLoading = $state(true);
	let isOpening = $state(false);
	let isClosing = $state(false);
	let errorMessage = $state<string | null>(null);
	let openErrorMessage = $state<string | null>(null);
	let closeErrorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	const activeRegisters = $derived(registers.filter((register) => register.status === 'active'));

	onMount(() => {
		void loadRegisterSessionState();
	});

	async function loadRegisterSessionState() {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;
		openErrorMessage = null;
		closeErrorMessage = null;

		try {
			const [registerResult, sessionResult] = await Promise.all([
				listCashRegisters({ limit: 100, status: ['active'] }),
				listCashRegisterSessions({ limit: 100, status: ['open'] })
			]);
			if (requestId !== latestRequestId) return;

			registers = registerResult.cashRegisters;
			openSessions = sessionResult.sessions;
			activeSession = resolveActiveSession(sessionResult.sessions);
			closedSession = null;

			if (activeSession) {
				setStoredActiveCashRegisterSessionId(activeSession.id);
				selectedCashRegisterId = String(activeSession.cash_register_id);
			} else {
				clearStoredActiveCashRegisterSessionId();
				selectedCashRegisterId = registers[0]?.id ? String(registers[0].id) : '';
			}
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error, t('point_flow.registerSession.errorLoad'));
			registers = [];
			openSessions = [];
			activeSession = null;
		} finally {
			if (requestId === latestRequestId) isLoading = false;
		}
	}

	async function handleOpenSession(event: SubmitEvent) {
		event.preventDefault();
		if (isOpening) return;

		const validationError = getOpenValidationError();
		if (validationError) {
			openErrorMessage = validationError;
			return;
		}

		isOpening = true;
		openErrorMessage = null;

		try {
			const session = await openCashRegisterSession({
				cash_register_id: Number(selectedCashRegisterId),
				opening_cash: toNumber(openingCash),
				opening_notes: emptyToNull(openingNotes)
			});
			activeSession = session;
			closedSession = null;
			closingCashCounted = '';
			closingNotes = '';
			setStoredActiveCashRegisterSessionId(session.id);
			toast.success(t('point_flow.registerSession.openedToast'), {
				description: t('point_flow.registerSession.openedToastDesc').replace(
					'{code}',
					session.cash_register_code
				)
			});
			await loadRegisterSessionState();
		} catch (error) {
			openErrorMessage = getErrorMessage(error, t('point_flow.registerSession.errorOpen'));
		} finally {
			isOpening = false;
		}
	}

	async function handleCloseSession(event: SubmitEvent) {
		event.preventDefault();
		if (!activeSession || isClosing) return;

		const validationError = getCloseValidationError();
		if (validationError) {
			closeErrorMessage = validationError;
			return;
		}

		isClosing = true;
		closeErrorMessage = null;

		try {
			const session = await closeCashRegisterSession(activeSession.id, {
				closing_cash_counted: toNumber(closingCashCounted),
				closing_notes: emptyToNull(closingNotes)
			});
			closedSession = session;
			activeSession = null;
			openSessions = openSessions.filter((item) => item.id !== session.id);
			clearStoredActiveCashRegisterSessionId();
			openingCash = '';
			openingNotes = '';
			closingCashCounted = '';
			closingNotes = '';
			toast.success(t('point_flow.registerSession.closedToast'), {
				description: t('point_flow.registerSession.closedToastDesc')
					.replace('{sessionNo}', session.session_no)
					.replace('{amount}', formatIdr(session.cash_difference))
			});
		} catch (error) {
			closeErrorMessage = getErrorMessage(error, t('point_flow.registerSession.errorClose'));
		} finally {
			isClosing = false;
		}
	}

	function chooseOpenSession(sessionId: string) {
		const session = openSessions.find((session) => String(session.id) === sessionId);
		if (!session) return;
		activeSession = session;
		closedSession = null;
		selectedCashRegisterId = String(session.cash_register_id);
		setStoredActiveCashRegisterSessionId(session.id);
	}

	function resolveActiveSession(sessions: CashRegisterSession[]) {
		const storedSessionId = getStoredActiveCashRegisterSessionId();
		if (storedSessionId) {
			const storedSession = sessions.find((session) => session.id === storedSessionId);
			if (storedSession) return storedSession;
		}
		return sessions[0] ?? null;
	}

	function getOpenValidationError() {
		if (!selectedCashRegisterId) return t('point_flow.registerSession.validNoRegister');
		if (!Number.isFinite(toNumber(openingCash)) || toNumber(openingCash) < 0) {
			return t('point_flow.registerSession.validOpeningCash');
		}
		return null;
	}

	function getCloseValidationError() {
		if (!Number.isFinite(toNumber(closingCashCounted)) || toNumber(closingCashCounted) < 0) {
			return t('point_flow.registerSession.validClosingCash');
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

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<svelte:head>
	<title>{t('point_flow.registerSession.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader
		title={t('point_flow.registerSession.title')}
		description={t('point_flow.registerSession.subtitle')}
	>
		{#snippet actions()}
			<div class="flex w-full flex-col gap-2 sm:w-auto sm:flex-row">
				<Button
					type="button"
					variant="outline"
					class="w-full sm:w-auto"
					onclick={loadRegisterSessionState}
				>
					<RefreshCwIcon />
					{t('common.refresh')}
				</Button>
				<Button class="w-full sm:w-auto" href="/point_flow/cash-drawer-movements">
					<BanknoteIcon />
					{t('point_flow.registerSession.cashMovementsBtn')}
				</Button>
			</div>
		{/snippet}
	</PageHeader>

	{#if isLoading}
		<div class="flex min-h-40 items-center justify-center gap-2 text-sm text-muted-foreground">
			<Spinner />
			{t('point_flow.registerSession.loadingSession')}
		</div>
	{:else if errorMessage}
		<ErrorAlert title={t('point_flow.registerSession.errorLoad')} message={errorMessage} />
	{:else}
		{#if openSessions.length > 1}
			<section class="grid gap-2">
				<Label>{t('point_flow.registerSession.activeSession')}</Label>
				<Select.Root
					allowDeselect={false}
					type="single"
					value={activeSession ? String(activeSession.id) : ''}
					onValueChange={chooseOpenSession}
				>
					<Select.Trigger class="w-full md:max-w-md">
						<Select.Value placeholder={t('point_flow.registerSession.selectSession')} />
					</Select.Trigger>
					<Select.Content>
						{#each openSessions as session (session.id)}
							<Select.Item
								value={String(session.id)}
								label={`${session.cash_register_code} - ${session.session_no}`}
							>
								{session.cash_register_code} - {session.session_no}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</section>
		{/if}

		{#if activeSession}
			<section class="grid gap-5 lg:grid-cols-[minmax(0,1fr)_420px]">
				<div class="grid gap-4">
					<div class="flex items-center gap-3 rounded-md border bg-muted/20 p-5">
						<div
							class="flex size-11 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
						>
							<UnlockIcon class="size-5" />
						</div>
						<div class="min-w-0">
							<h2 class="text-base font-semibold">{t('point_flow.registerSession.openSection')}</h2>
							<p class="truncate text-sm text-muted-foreground">
								{activeSession.cash_register_code} - {activeSession.cash_register_name}
							</p>
						</div>
					</div>

					<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
						<DetailField
							label={t('point_flow.registerSession.sessionNo')}
							value={activeSession.session_no}
						/>
						<DetailField
							label={t('point_flow.registerSession.register')}
							value={activeSession.cash_register_code}
						/>
						<DetailField
							label={t('point_flow.registerSession.openedBy')}
							value={activeSession.opened_by_name}
						/>
						<DetailField
							label={t('point_flow.registerSession.openedAt')}
							value={formatDateTime(activeSession.opened_at)}
						/>
						<DetailField
							label={t('point_flow.registerSession.openingCash')}
							value={formatIdr(activeSession.opening_cash)}
						/>
						<DetailField
							label={t('point_flow.registerSession.openingNotes')}
							value={activeSession.opening_notes ?? '-'}
						/>
					</div>
				</div>

				<form class="grid content-start gap-4 rounded-md border p-5" onsubmit={handleCloseSession}>
					<div class="flex items-center gap-2">
						<LockIcon class="size-5 text-muted-foreground" />
						<h2 class="text-base font-semibold">{t('point_flow.registerSession.closeSection')}</h2>
					</div>

					<div class="grid gap-2">
						<Label for="closing-cash-counted">{t('point_flow.registerSession.closingCash')}</Label>
						<Input
							id="closing-cash-counted"
							type="number"
							min="0"
							step="1"
							placeholder="0"
							disabled={isClosing}
							bind:value={closingCashCounted}
						/>
					</div>

					<div
						class="grid gap-2 rounded-md bg-muted/30 p-3 text-sm"
						aria-label={t('point_flow.registerSession.closeSummary')}
					>
						<div class="flex justify-between gap-3">
							<span class="text-muted-foreground">{t('point_flow.registerSession.openingCash')}</span>
							<span class="font-medium">{formatIdr(activeSession.opening_cash)}</span>
						</div>
						<p class="text-muted-foreground">{t('point_flow.registerSession.expectedCashDesc')}</p>
					</div>

					<div class="grid gap-2">
						<Label for="closing-notes">{t('point_flow.registerSession.closingNotes')}</Label>
						<Textarea
							id="closing-notes"
							placeholder={t('point_flow.registerSession.closingNotesPlaceholder')}
							disabled={isClosing}
							bind:value={closingNotes}
						/>
					</div>

					{#if closeErrorMessage}
						<ErrorAlert title={t('point_flow.registerSession.errorClose')} message={closeErrorMessage} />
					{/if}

					<Button type="submit" disabled={isClosing}>
						{#if isClosing}
							<Spinner />
						{:else}
							<LockIcon />
						{/if}
						{isClosing ? t('point_flow.registerSession.closing') : t('point_flow.registerSession.closeBtn')}
					</Button>
				</form>
			</section>
		{:else}
			<section class="grid gap-5 lg:grid-cols-[minmax(0,1fr)_420px]">
				<div class="grid content-start gap-4">
					<div class="flex items-center gap-3 rounded-md border bg-muted/20 p-5">
						<div
							class="flex size-11 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
						>
							<CircleDollarSignIcon class="size-5" />
						</div>
						<div class="min-w-0">
							<h2 class="text-base font-semibold">{t('point_flow.registerSession.noActiveRegister')}</h2>
							<p class="text-sm text-muted-foreground">
								{t('point_flow.registerSession.noActiveRegisterDesc')}
							</p>
						</div>
					</div>

					{#if closedSession}
						<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
							<DetailField
								label={t('point_flow.registerSession.closedSession')}
								value={closedSession.session_no}
							/>
							<DetailField
								label={t('point_flow.registerSession.expectedCash')}
								value={formatIdr(closedSession.expected_cash)}
							/>
							<DetailField
								label={t('point_flow.registerSession.countedCash')}
								value={formatIdr(closedSession.closing_cash_counted)}
							/>
							<DetailField
								label={t('point_flow.registerSession.difference')}
								value={formatIdr(closedSession.cash_difference)}
							/>
						</div>
					{/if}
				</div>

				<form class="grid content-start gap-4 rounded-md border p-5" onsubmit={handleOpenSession}>
					<div class="flex items-center gap-2">
						<UnlockIcon class="size-5 text-muted-foreground" />
						<h2 class="text-base font-semibold">{t('point_flow.registerSession.openBtn')}</h2>
					</div>

					<div class="grid gap-2">
						<Label>{t('point_flow.registerSession.cashRegister')}</Label>
						<Select.Root
							allowDeselect={false}
							type="single"
							value={selectedCashRegisterId}
							disabled={isOpening || activeRegisters.length === 0}
							onValueChange={(value) => (selectedCashRegisterId = value)}
						>
							<Select.Trigger class="w-full">
								<Select.Value placeholder={t('point_flow.registerSession.selectRegister')} />
							</Select.Trigger>
							<Select.Content>
								{#each activeRegisters as register (register.id)}
									<Select.Item
										value={String(register.id)}
										label={`${register.code} - ${register.name}`}
									>
										{register.code} - {register.name}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<div class="grid gap-2">
						<Label for="opening-cash">{t('point_flow.registerSession.openingCash')}</Label>
						<Input
							id="opening-cash"
							type="number"
							min="0"
							step="1"
							placeholder="0"
							disabled={isOpening}
							bind:value={openingCash}
						/>
					</div>

					<div class="grid gap-2">
						<Label for="opening-notes">{t('point_flow.registerSession.openingNotes')}</Label>
						<Textarea
							id="opening-notes"
							placeholder={t('point_flow.registerSession.openingNotesPlaceholder')}
							disabled={isOpening}
							bind:value={openingNotes}
						/>
					</div>

					{#if activeRegisters.length === 0}
						<ErrorAlert
							title={t('point_flow.registerSession.noActiveRegisterError')}
							message={t('point_flow.registerSession.noRegisterSetup')}
						/>
					{/if}
					{#if openErrorMessage}
						<ErrorAlert title={t('point_flow.registerSession.errorOpen')} message={openErrorMessage} />
					{/if}

					<Button type="submit" disabled={isOpening || activeRegisters.length === 0}>
						{#if isOpening}
							<Spinner />
						{:else}
							<UnlockIcon />
						{/if}
						{isOpening ? t('point_flow.registerSession.opening') : t('point_flow.registerSession.openBtn')}
					</Button>
				</form>
			</section>
		{/if}
	{/if}
</div>
