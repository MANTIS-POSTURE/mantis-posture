<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';

	function minimize() {
		getCurrentWindow().minimize();
	}

	function toggleMaximize() {
		getCurrentWindow().toggleMaximize();
	}

	function closeWindow() {
		getCurrentWindow().close();
	}
</script>

<header class="titlebar" data-tauri-drag-region>
	<div class="titlebar-brand">
		<span class="brand-mark">M</span>
		<span class="brand-name">MANTIS POSTURE</span>
	</div>
	<div class="titlebar-phase">Phase 1 — Cockpit</div>
	<div class="titlebar-controls">
		<button class="ctrl-btn minimize" onclick={minimize} aria-label="Réduire" title="Réduire">
			<svg viewBox="0 0 12 12" width="12" height="12"><rect x="1" y="5.5" width="10" height="1" fill="currentColor"/></svg>
		</button>
		<button class="ctrl-btn maximize" onclick={toggleMaximize} aria-label="Agrandir" title="Agrandir">
			<svg viewBox="0 0 12 12" width="12" height="12"><rect x="1.5" y="1.5" width="9" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
		</button>
		<button class="ctrl-btn close" onclick={closeWindow} aria-label="Fermer" title="Fermer">
			<svg viewBox="0 0 12 12" width="12" height="12"><path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.2"/></svg>
		</button>
	</div>
</header>

<style>
	.titlebar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 38px;
		padding: 0 0.75rem;
		background: var(--mantis-bg-sidebar);
		border-bottom: 1px solid var(--mantis-border);
		-webkit-app-region: drag;
		app-region: drag;
		user-select: none;
	}

	.titlebar-brand {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.brand-mark {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border: 1px solid var(--mantis-border);
		border-radius: 5px;
		background: var(--mantis-bg-raised);
		color: var(--mantis-accent);
		font-size: 0.7rem;
		font-weight: 700;
	}

	.brand-name {
		font-size: 0.78rem;
		font-weight: 700;
		letter-spacing: 0.1em;
	}

	.titlebar-phase {
		font-size: 0.7rem;
		color: var(--mantis-text-muted);
		letter-spacing: 0.05em;
	}

	.titlebar-controls {
		display: flex;
		gap: 2px;
		-webkit-app-region: no-drag;
		app-region: no-drag;
	}

	.ctrl-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 28px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--mantis-text-muted);
		cursor: pointer;
		transition: background 0.15s, color 0.15s;
	}

	.ctrl-btn:hover {
		background: var(--mantis-bg-raised);
		color: var(--mantis-text);
	}

	.ctrl-btn.close:hover {
		background: var(--mantis-danger);
		color: #fff;
	}
</style>