<script lang="ts">
	import type { Snippet } from 'svelte';
	import { t } from '$lib/i18n';
	import { tick } from 'svelte';

	let { open, title, description, onClose, children, actions, dismissible = true, wide = false }: { open: boolean; title: string; description?: string; onClose: () => void; children?: Snippet; actions?: Snippet; dismissible?: boolean; wide?: boolean } = $props();
	let modalElement = $state<HTMLDivElement>();
	let previouslyFocused: HTMLElement | null = null;

	function focusableElements() {
		return modalElement?.querySelectorAll<HTMLElement>('button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])') ?? [];
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!open) return;
		if (event.key === 'Escape') { if (!dismissible) return; event.preventDefault(); onClose(); return; }
		if (event.key !== 'Tab') return;
		const elements = focusableElements();
		if (!elements.length) { event.preventDefault(); return; }
		const first = elements[0];
		const last = elements[elements.length - 1];
		if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
		else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
	}

	$effect(() => {
		if (!open) {
			previouslyFocused?.focus();
			previouslyFocused = null;
			return;
		}
		previouslyFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null;
		void tick().then(() => focusableElements()[0]?.focus());
	});
</script>

<svelte:window onkeydown={handleKeydown} />
{#if open}
	<div class="modal-layer">
		<div class:wide class="modal" bind:this={modalElement} role="dialog" aria-modal="true" aria-labelledby="modal-title" aria-describedby={description ? 'modal-description' : undefined}>
			{#if dismissible}<button class="modal-close" type="button" aria-label={t('Fermer')} onclick={onClose}>×</button>{/if}
			<header><h2 id="modal-title">{t(title)}</h2>{#if description}<p id="modal-description">{t(description)}</p>{/if}</header>
			{#if children}<div class="modal-content">{@render children()}</div>{/if}
			{#if actions}<footer>{@render actions()}</footer>{/if}
		</div>
	</div>
{/if}

<style>
	.modal-layer { position:fixed; z-index:1000; inset:0; display:grid; place-items:center; padding:1rem; background:rgba(0,0,0,.78); backdrop-filter:blur(var(--blur-overlay)) saturate(85%); -webkit-backdrop-filter:blur(var(--blur-overlay)) saturate(85%); }
	.modal { position:relative; width:min(480px,100%); max-height:min(760px,calc(100vh - 2rem)); overflow:auto; padding:1.35rem; border:1px solid var(--ui-border-strong); border-radius:var(--radius-lg); background:var(--ui-material-overlay); box-shadow:inset 0 1px 0 var(--ui-rim-light),var(--shadow-overlay); }
	.modal.wide { width:min(1040px,calc(100vw - 2rem)); max-height:min(860px,calc(100vh - 2rem)); padding:1.6rem; }
	header { padding-right:2rem; } h2 { margin:0; font-size:1.2rem; } header p { margin:.45rem 0 0; color:var(--ui-text-secondary); font-size:.85rem; line-height:1.5; }.modal-content { margin-top:1rem; } footer { display:flex; flex-wrap:wrap; justify-content:flex-end; gap:.5rem; margin-top:1.15rem; padding-top:1rem; border-top:1px solid var(--ui-border-subtle); }
	.modal-close { position:absolute; top:.75rem; right:.75rem; width:34px; height:34px; border:1px solid transparent; border-radius:var(--radius-sm); color:var(--ui-text-secondary); background:transparent; font-size:1.3rem; cursor:pointer; }.modal-close:hover { color:var(--ui-text-primary); border-color:var(--ui-border-subtle); background:var(--ui-surface-3); }
	@media (prefers-reduced-transparency: reduce) { .modal-layer { backdrop-filter:none; -webkit-backdrop-filter:none; } }
</style>
