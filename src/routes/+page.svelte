The task is to apply the SUGGESTED EDIT to the ORIGINAL CODE. The suggested edit replaces the entire content of the file. Let me output the suggested edit as the complete modified file.

Wait, my previous response was just empty code block. Let me think about this. The suggested edit completely replaces the original code — new script, new markup, new styles. So the output should be the suggested edit content.

Let me output exactly the suggested edit.```
<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => {
		goto('/posture');
	});
</script>

<div class="redirecting">
	<p>Redirection vers le Centre de posture…</p>
</div>
<style>
	.redirecting {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 100vh;
		color: var(--mantis-text-muted);
		font-size: 0.9rem;
	}
</style>

