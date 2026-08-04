<script lang="ts">
	let { title, message, href, actionLabel, symbol = '◇' }: { title: string; message?: string; href?: string; actionLabel?: string; symbol?: string } = $props();
</script>

<div class="empty-state">
	<span class="empty-symbol" aria-hidden="true">{symbol}</span>
	<strong>{title}</strong>
	{#if message}<p>{message}</p>{/if}
	{#if href && actionLabel}<a class="wf-btn primary" {href}>{actionLabel}</a>{/if}
</div>

<style>
	.empty-state { display:grid; justify-items:center; gap:.45rem; padding:clamp(1.5rem,4vw,3rem) 1rem; text-align:center; border:1px dashed var(--ui-border-default); border-radius:var(--radius-md); background:color-mix(in srgb,var(--ui-surface-1) 82%,transparent); }
	.empty-symbol { display:grid; place-items:center; width:42px; height:42px; margin-bottom:.25rem; border:1px solid color-mix(in srgb,var(--ui-accent) 40%,var(--ui-border-default)); border-radius:50%; color:var(--ui-accent); background:var(--ui-accent-soft); }
	strong { color:var(--ui-text-primary); font-size:.95rem; } p { max-width:34rem; margin:0 0 .35rem; color:var(--ui-text-secondary); font-size:.84rem; line-height:1.5; }
</style>
