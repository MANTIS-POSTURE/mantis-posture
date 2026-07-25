<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Short label above the CTA, e.g. "Ensuite" */
		label?: string;
		/** Plain-language next step explanation */
		hint: string;
		/** Primary link CTA */
		primaryHref?: string;
		primaryLabel?: string;
		/** Optional primary button (mutually preferred with href when both set: href wins for nav) */
		onPrimary?: () => void;
		primaryDisabled?: boolean;
		children?: Snippet;
	}

	let {
		label = 'Ensuite',
		hint,
		primaryHref,
		primaryLabel,
		onPrimary,
		primaryDisabled = false,
		children
	}: Props = $props();
</script>

<div class="next-bar">
	<div class="next-copy">
		<span class="next-label">{label}</span>
		<p class="next-hint">{hint}</p>
	</div>
	<div class="next-ctas">
		{#if primaryHref && primaryLabel}
			<a class="wf-btn primary" href={primaryHref}>{primaryLabel}</a>
		{:else if onPrimary && primaryLabel}
			<button
				type="button"
				class="wf-btn primary"
				disabled={primaryDisabled}
				onclick={onPrimary}
			>
				{primaryLabel}
			</button>
		{/if}
		{#if children}
			{@render children()}
		{/if}
	</div>
</div>

<style>
	.next-bar {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: 0.85rem;
		margin-top: 1.1rem;
		padding: 0.9rem 1rem;
		border-radius: 8px;
		border: 1px solid color-mix(in srgb, var(--mantis-accent) 35%, var(--mantis-border));
		background: color-mix(in srgb, var(--mantis-accent) 8%, var(--mantis-bg));
	}

	.next-label {
		display: block;
		font-size: 0.68rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--mantis-accent);
		margin-bottom: 0.25rem;
	}

	.next-hint {
		margin: 0;
		font-size: 0.88rem;
		max-width: 36rem;
		line-height: 1.4;
	}

	.next-ctas {
		display: flex;
		flex-wrap: wrap;
		gap: 0.45rem;
		align-items: center;
	}
</style>
