<script lang="ts">
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import MailIcon from '@lucide/svelte/icons/mail';
	import MessageCircleIcon from '@lucide/svelte/icons/message-circle';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import { localeState, t } from '$lib/i18n/locale.svelte.js';

	// Support channels and version — placeholders, swap for the deployment's real values.
	const SUPPORT_EMAIL = 'support@pointflow.app';
	const SUPPORT_WHATSAPP = '6281234567890';
	const APP_VERSION = '0.0.1';

	const CATEGORIES = [
		'all',
		'pos',
		'procurement',
		'inventory',
		'payroll',
		'reports',
		'access'
	] as const;

	// FAQ entries: structure lives here, copy lives in i18n (`helpDesk.faq.<key>.q|a`).
	const FAQ_ITEMS = [
		{ key: 'openSession', category: 'pos' },
		{ key: 'noSession', category: 'pos' },
		{ key: 'saleReturn', category: 'pos' },
		{ key: 'cashMovement', category: 'pos' },
		{ key: 'poToGr', category: 'procurement' },
		{ key: 'postGr', category: 'procurement' },
		{ key: 'stockAdjust', category: 'inventory' },
		{ key: 'stockConvert', category: 'inventory' },
		{ key: 'stockChanges', category: 'inventory' },
		{ key: 'payrollSlip', category: 'payroll' },
		{ key: 'salesReports', category: 'reports' },
		{ key: 'permissionDenied', category: 'access' },
		{ key: 'languageTheme', category: 'access' },
		{ key: 'changePassword', category: 'access' },
		{ key: 'documentLifecycle', category: 'access' }
	] as const;

	let query = $state('');
	let activeCategory = $state<(typeof CATEGORIES)[number]>('all');

	const normalizedQuery = $derived(query.trim().toLowerCase());

	// Filter by active category, then by a case-insensitive match on the question + answer.
	// Reads `t()` so the list re-filters when the locale changes.
	const filtered = $derived(
		FAQ_ITEMS.filter((item) => {
			if (activeCategory !== 'all' && item.category !== activeCategory) return false;
			if (!normalizedQuery) return true;
			const question = t(`helpDesk.faq.${item.key}.q`).toLowerCase();
			const answer = t(`helpDesk.faq.${item.key}.a`).toLowerCase();
			return question.includes(normalizedQuery) || answer.includes(normalizedQuery);
		})
	);
</script>

<svelte:head>
	<title>{t('helpDesk.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader title={t('helpDesk.title')} description={t('helpDesk.subtitle')} />

	<div class="flex flex-col gap-4">
		<SearchField
			value={query}
			placeholder={t('helpDesk.searchPlaceholder')}
			ariaLabel={t('helpDesk.searchAriaLabel')}
			onValueChange={(value) => (query = value)}
		/>

		<div class="flex flex-wrap gap-2">
			{#each CATEGORIES as category (category)}
				<Button
					variant={activeCategory === category ? 'default' : 'outline'}
					size="sm"
					onclick={() => (activeCategory = category)}
				>
					{t(`helpDesk.categories.${category}`)}
				</Button>
			{/each}
		</div>
	</div>

	{#if filtered.length === 0}
		<div
			class="flex items-center justify-center rounded-md border border-dashed py-12 text-sm text-muted-foreground"
		>
			{t('helpDesk.noResults')}
		</div>
	{:else}
		<div class="flex flex-col gap-2">
			{#each filtered as item (item.key)}
				<Collapsible.Root class="overflow-hidden rounded-lg border border-border/60 bg-card">
					<Collapsible.Trigger
						class="group flex w-full items-center justify-between gap-4 px-4 py-3 text-left text-sm font-medium transition-colors hover:bg-muted/50"
					>
						{t(`helpDesk.faq.${item.key}.q`)}
						<ChevronDownIcon
							class="size-4 shrink-0 text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180"
						/>
					</Collapsible.Trigger>
					<Collapsible.Content>
						<p class="px-4 pb-4 text-sm leading-relaxed text-muted-foreground">
							{t(`helpDesk.faq.${item.key}.a`)}
						</p>
					</Collapsible.Content>
				</Collapsible.Root>
			{/each}
		</div>
	{/if}

	<div class="grid gap-4 md:grid-cols-2">
		<Card.Root class="border border-border/40">
			<Card.Header>
				<Card.Title class="text-base">{t('helpDesk.contactTitle')}</Card.Title>
				<Card.Description>{t('helpDesk.contactDesc')}</Card.Description>
			</Card.Header>
			<Card.Content class="flex flex-col gap-2 sm:flex-row">
				<a class={buttonVariants({ variant: 'outline' })} href={`mailto:${SUPPORT_EMAIL}`}>
					<MailIcon class="size-4" />
					{t('helpDesk.contactEmail')}
				</a>
				<a
					class={buttonVariants({ variant: 'outline' })}
					href={`https://wa.me/${SUPPORT_WHATSAPP}`}
					target="_blank"
					rel="noopener noreferrer"
				>
					<MessageCircleIcon class="size-4" />
					{t('helpDesk.contactWhatsapp')}
				</a>
			</Card.Content>
		</Card.Root>

		<Card.Root class="border border-border/40">
			<Card.Header>
				<Card.Title class="text-base">{t('helpDesk.appInfoTitle')}</Card.Title>
			</Card.Header>
			<Card.Content class="flex flex-col gap-2 text-sm">
				<div class="flex items-center justify-between">
					<span class="text-muted-foreground">{t('helpDesk.appInfoVersion')}</span>
					<span class="font-mono tabular-nums">{APP_VERSION}</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-muted-foreground">{t('helpDesk.appInfoLanguage')}</span>
					<span class="font-mono">{localeState.locale.toUpperCase()}</span>
				</div>
			</Card.Content>
		</Card.Root>
	</div>
</div>
