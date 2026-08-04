<script lang="ts">
	import { t } from '$lib/i18n';
	let { label, tone = 'neutral', dot = false }: { label: string; tone?: 'neutral'|'accent'|'info'|'success'|'warning'|'danger'|'ai'; dot?: boolean } = $props();
</script>

<span class="status-badge tone-{tone}">{#if dot}<i aria-hidden="true"></i>{/if}{t(label)}</span>

<style>
	.status-badge { --tone:var(--ui-text-secondary); display:inline-flex; align-items:center; gap:.35rem; width:max-content; max-width:100%; padding:.24rem .52rem; border:1px solid color-mix(in srgb,var(--tone) 42%,var(--ui-border-default)); border-radius:var(--radius-pill); color:var(--tone); background:color-mix(in srgb,var(--tone) 10%,var(--ui-surface-1)); font-family:var(--font-meta); font-size:.68rem; font-weight:700; line-height:1.2; letter-spacing:.045em; overflow-wrap:anywhere; }
	i { width:6px; height:6px; flex:0 0 auto; border-radius:50%; background:currentColor; box-shadow:0 0 5px color-mix(in srgb,currentColor 38%,transparent); }
	.tone-accent { --tone:var(--ui-accent); }.tone-info { --tone:var(--ui-info); }.tone-success { --tone:var(--ui-success); }.tone-warning { --tone:var(--ui-warning); }.tone-danger { --tone:var(--ui-danger); }.tone-ai { --tone:var(--ui-ai); }
</style>
