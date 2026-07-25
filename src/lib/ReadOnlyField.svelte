<script lang="ts">
	interface Props {
		label: string;
		value: string;
		hint?: string;
	}

	let {
		label,
		value,
		hint = 'Non éditable pour l’instant — saisie prévue en Phase 2 (base locale).'
	}: Props = $props();
</script>

<div class="ro-field">
	<div class="ro-top">
		<span class="ro-label">{label}</span>
		<span class="ro-badge" title={hint}>Lecture seule</span>
	</div>
	<p class="ro-value">{value}</p>
	<p class="ro-hint">{hint}</p>
</div>

<style>
	.ro-field {
		margin: 0 0 0.9rem;
		padding: 0.7rem 0.85rem;
		border-radius: 8px;
		border: 1px dashed var(--mantis-border);
		background: color-mix(in srgb, var(--mantis-bg) 70%, transparent);
	}

	.ro-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		margin-bottom: 0.3rem;
	}

	.ro-label {
		font-size: 0.72rem;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--mantis-text-muted);
	}

	.ro-badge {
		flex-shrink: 0;
		padding: 0.12rem 0.4rem;
		border-radius: 4px;
		font-size: 0.65rem;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--mantis-text-muted);
		border: 1px solid var(--mantis-border);
	}

	.ro-value {
		margin: 0;
		font-size: 0.9rem;
		word-break: break-word;
	}

	.ro-hint {
		margin: 0.35rem 0 0;
		font-size: 0.72rem;
		color: var(--mantis-text-muted);
		line-height: 1.4;
	}
</style>
