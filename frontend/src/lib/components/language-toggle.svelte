<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { localeState, setLocale, t } from '$lib/i18n/locale.svelte.js';
	import type { Locale } from '$lib/i18n/messages.js';
	import { cn } from '$lib/utils.js';

	// Language names are autonyms — shown in their own language regardless of UI locale.
	const options: { value: Locale; label: string }[] = [
		{ value: 'id', label: 'Bahasa Indonesia' },
		{ value: 'en', label: 'English' }
	];
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				type="button"
				variant="ghost"
				size="icon-sm"
				aria-label={t('language.label')}
			>
				<LanguagesIcon aria-hidden="true" />
				<span class="sr-only">{t('language.label')}</span>
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" class="min-w-44">
		<DropdownMenu.Label>{t('language.label')}</DropdownMenu.Label>
		<DropdownMenu.Separator />
		{#each options as option (option.value)}
			<DropdownMenu.Item onSelect={() => setLocale(option.value)}>
				<CheckIcon
					class={cn('size-4', localeState.locale === option.value ? 'opacity-100' : 'opacity-0')}
				/>
				{option.label}
			</DropdownMenu.Item>
		{/each}
	</DropdownMenu.Content>
</DropdownMenu.Root>
