<script lang="ts">
	let { name }: { name: string } = $props();
</script>

<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
	{#if name === 'posture'}
		<path d="M4 13a8 8 0 1 1 16 0"/><path d="m12 13 4-4"/><path d="M5 18h14"/>
	{:else if name === 'reports'}
		<path d="M5 3h10l4 4v14H5z"/><path d="M15 3v5h5M8 13h8M8 17h6"/>
	{:else if name === 'folders'}
		<path d="M3 6h7l2 2h9v11H3z"/>
	{:else if name === 'identities'}
		<circle cx="12" cy="8" r="3.5"/><path d="M5.5 20a6.5 6.5 0 0 1 13 0"/>
	{:else if name === 'watch'}
		<path d="M2.5 12s3.5-6 9.5-6 9.5 6 9.5 6-3.5 6-9.5 6-9.5-6-9.5-6Z"/><circle cx="12" cy="12" r="2.5"/>
	{:else if name === 'graph'}
		<circle cx="5" cy="12" r="2"/><circle cx="18" cy="6" r="2"/><circle cx="19" cy="18" r="2"/><path d="m7 11 9-4M7 13l10 4M18 8l1 8"/>
	{:else if name === 'exposures'}
		<path d="M12 3 2.8 19h18.4Z"/><path d="M12 9v4M12 17h.01"/>
	{:else if name === 'incidents'}
		<path d="M12 22a9 9 0 1 0-9-9"/><path d="M3 4v5h5M12 8v5l3 2"/>
	{:else if name === 'actions'}
		<path d="M5 12.5 9.5 17 19 7.5"/>
	{:else if name === 'dpo'}
		<path d="M12 3 5 6v5c0 4.6 2.8 8 7 10 4.2-2 7-5.4 7-10V6Z"/><path d="M9 12h6M12 9v6"/>
	{:else if name === 'guides'}
		<path d="M4 5.5A3.5 3.5 0 0 1 7.5 2H12v18H7.5A3.5 3.5 0 0 0 4 23Z"/><path d="M20 5.5A3.5 3.5 0 0 0 16.5 2H12v18h4.5A3.5 3.5 0 0 1 20 23Z"/>
	{:else}
		<circle cx="12" cy="12" r="3"/><path d="M19 15.5a2 2 0 0 0 .4 2.2l-1.7 1.7a2 2 0 0 0-2.2-.4 2 2 0 0 0-1.2 1.8h-2.5a2 2 0 0 0-1.3-1.8 2 2 0 0 0-2.2.4l-1.7-1.7a2 2 0 0 0 .4-2.2 2 2 0 0 0-1.8-1.2v-2.5A2 2 0 0 0 7 10.5a2 2 0 0 0-.4-2.2l1.7-1.7a2 2 0 0 0 2.2.4 2 2 0 0 0 1.3-1.8h2.5A2 2 0 0 0 15.5 7a2 2 0 0 0 2.2-.4l1.7 1.7a2 2 0 0 0-.4 2.2 2 2 0 0 0 1.8 1.3v2.5a2 2 0 0 0-1.8 1.2Z"/>
	{/if}
</svg>

<style>
	svg { width: 18px; height: 18px; display: block; }
</style>
