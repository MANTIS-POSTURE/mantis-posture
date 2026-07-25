<script lang="ts">
  import { onMount } from 'svelte';
  import { listIdentities, type Identity } from '$lib/api';

  let identities: Identity[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      identities = await listIdentities();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
  <h2 class="text-xl font-bold mb-4 text-slate-100">Identités</h2>
  
  {#if loading}
    <p class="text-slate-400">Chargement...</p>
  {:else if error}
    <p class="text-red-400">Erreur: {error}</p>
  {:else if identities.length === 0}
    <p class="text-slate-400">Aucune identité enregistrée.</p>
  {:else}
    <div class="overflow-x-auto">
      <table class="min-w-full text-sm text-left text-slate-300">
        <thead class="text-xs uppercase bg-slate-700/50 text-slate-400">
          <tr>
            <th class="px-4 py-3">Type</th>
            <th class="px-4 py-3">Valeur</th>
            <th class="px-4 py-3">Label</th>
          </tr>
        </thead>
        <tbody>
          {#each identities as identity (identity.id)}
            <tr class="border-b border-slate-700">
              <td class="px-4 py-2 capitalize">{identity.identity_type}</td>
              <td class="px-4 py-2 font-mono text-slate-100">{identity.value}</td>
              <td class="px-4 py-2 text-slate-400">{identity.label || '-'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
