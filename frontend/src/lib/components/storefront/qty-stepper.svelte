<script lang="ts">
	type Props = {
		quantity: number;
		max?: number;
		onchange?: (quantity: number) => void;
	};

	let { quantity, max, onchange }: Props = $props();

	function decrease() {
		onchange?.(quantity - 1);
	}

	function increase() {
		if (max !== undefined && quantity >= max) return;
		onchange?.(quantity + 1);
	}
</script>

<div class="inline-flex items-center gap-1 rounded-full bg-neutral-100 px-1 py-1">
	<button
		type="button"
		onclick={decrease}
		aria-label="−"
		class="flex size-7 items-center justify-center rounded-full text-lg font-bold text-[#333333] transition-colors hover:bg-white"
	>
		−
	</button>
	<span class="min-w-7 text-center text-sm font-bold text-[#333333]" aria-live="polite">
		{quantity}
	</span>
	<button
		type="button"
		onclick={increase}
		disabled={max !== undefined && quantity >= max}
		aria-label="+"
		class="flex size-7 items-center justify-center rounded-full text-lg font-bold text-[#333333] transition-colors hover:bg-white disabled:cursor-not-allowed disabled:opacity-40"
	>
		+
	</button>
</div>
