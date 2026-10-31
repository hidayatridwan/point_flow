<script lang="ts">
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		name,
		email,
		actorId
	}: {
		name: string | null;
		email: string | null;
		actorId: number | null;
	} = $props();
</script>

<div class="grid min-w-40 gap-0.5">
	<div class="truncate font-medium">{name || t('common.systemActor')}</div>
	<div class="truncate text-xs text-muted-foreground">
		{email ||
			(actorId
				? t('config.auditLogs.actorLabel').replace('{id}', String(actorId))
				: t('common.noActor'))}
	</div>
</div>
