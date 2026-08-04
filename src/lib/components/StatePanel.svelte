<script lang="ts">
	import { t } from '$lib/i18n';
	interface Props { tone?: 'neutral'|'info'|'success'|'warning'|'danger'|'ai'; title: string; message?: string; compact?: boolean; live?: 'off'|'polite'|'assertive'; }
	let { tone='neutral', title, message, compact=false, live='off' }: Props = $props();
</script>

<div class:compact class="state-panel tone-{tone}" role={tone === 'danger' ? 'alert' : 'status'} aria-live={live}>
	<span class="state-icon" aria-hidden="true">{tone === 'success' ? '✓' : tone === 'danger' ? '!' : tone === 'warning' ? '!' : tone === 'ai' ? '✦' : tone === 'info' ? 'i' : '·'}</span>
	<div><strong>{t(title)}</strong>{#if message}<p>{t(message)}</p>{/if}</div>
</div>

<style>
	.state-panel { --tone:var(--ui-text-secondary); --soft:var(--ui-material-solid); display:flex; align-items:flex-start; gap:.7rem; padding:.85rem 1rem; border:1px solid color-mix(in srgb,var(--tone) 36%,var(--ui-border-default)); border-left:3px solid var(--tone); border-radius:var(--radius-md); background:color-mix(in srgb,var(--tone) 7%,var(--soft)); box-shadow:inset 0 1px 0 var(--ui-rim-light); }
	.state-panel.compact { padding:.65rem .75rem; }
	.state-icon { display:grid; place-items:center; width:22px; height:22px; flex:0 0 auto; border:1px solid color-mix(in srgb,var(--tone) 45%,transparent); border-radius:50%; color:var(--tone); background:color-mix(in srgb,var(--tone) 10%,transparent); font-size:.72rem; font-weight:800; }
	strong { display:block; color:var(--ui-text-primary); font-size:.86rem; } p { margin:.18rem 0 0; color:var(--ui-text-secondary); font-size:.8rem; line-height:1.5; overflow-wrap:anywhere; }
	.tone-info { --tone:var(--ui-info); }.tone-success { --tone:var(--ui-success); }.tone-warning { --tone:var(--ui-warning); }.tone-danger { --tone:var(--ui-danger); }.tone-ai { --tone:var(--ui-ai); }
</style>
