<script lang="ts">
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import * as Alert from '$lib/components/ui/alert/index.js';

	let {
		title,
		message
	}: {
		title: string;
		message: string;
	} = $props();
</script>

<Alert.Root variant="destructive">
	<CircleAlertIcon />
	<Alert.Title>{title}</Alert.Title>
	<Alert.Description>{message}</Alert.Description>
</Alert.Root>
