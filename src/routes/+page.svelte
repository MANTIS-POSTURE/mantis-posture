<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  onMount(() => {
    goto('/posture', { replaceState: true });
  });
</script>

<div class="loading-screen">
  <p>Chargement du cockpit...</p>
</div>

<style>
  .loading-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    width: 100%;
    color: var(--mantis-text-muted);
    font-family: system-ui, sans-serif;
  }
</style>
