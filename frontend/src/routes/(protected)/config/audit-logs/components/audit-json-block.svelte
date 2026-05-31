<script lang="ts">
	import type { AuditJsonValue } from '$lib/api/audit-logs.js';

	let {
		label,
		value
	}: {
		label: string;
		value: AuditJsonValue | null;
	} = $props();

	const formattedValue = $derived(formatJson(value));

	function formatJson(value: AuditJsonValue | null) {
		return JSON.stringify(value, null, 2);
	}
</script>

<section class="grid gap-2">
	<h3 class="text-sm font-medium">{label}</h3>
	<pre
		class="max-h-72 overflow-auto rounded-md border bg-muted/30 p-3 text-xs leading-relaxed"><code
			>{formattedValue}</code
		></pre>
</section>
