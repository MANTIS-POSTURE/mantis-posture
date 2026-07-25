<script lang="ts">
  import { onMount } from 'svelte';
  import { listIncidents, listActions, type Incident, type Action } from '$lib/api';

  let incidents: Incident[] = [];
  let actions: Action[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      incidents = await listIncidents();
      actions = await listActions();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function getActionsForIncident(incidentId: number): Action[] {
    return actions.filter(a => a.incident_id === incidentId);
  }
</script>

<div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
  <h2 class="text-xl font-bold mb-4 text-slate-100">Incidents & Actions</h2>
  
  {#if loading}
    <p class="text-slate-400">Chargement...</p>
  {:else if error}
    <p class="text-red-400">Erreur: {error}</p>
  {:else if incidents.length === 0}
    <p class="text-slate-400">Aucun incident enregistré.</p>
  {:else}
    <div class="space-y-4">
      {#each incidents as incident (incident.id)}
        <div class="border border-slate-700 rounded-md p-3 bg-slate-900/50">
          <div class="flex justify-between items-start mb-2">
            <h3 class="text-lg font-semibold text-slate-100">{incident.title}</h3>
            <span class="px-2 py-1 text-xs rounded-full bg-red-900/50 text-red-300 border border-red-700">
              {incident.severity}
            </span>
          </div>
          <p class="text-sm text-slate-400 mb-3">{incident.description}</p>
          
          <div class="mt-2">
            <h4 class="text-sm font-medium text-slate-300 mb-1">Actions requises:</h4>
            <ul class="space-y-1">
              {#each getActionsForIncident(incident.id) as action (action.id)}
                <li class="flex items-center text-sm text-slate-300">
                  <span class="mr-2 text-slate-500">•</span>
                  {action.title}
                  <span class="ml-auto text-xs px-2 py-0.5 rounded bg-slate-700 text-slate-300">{action.status}</span>
                </li>
              {/each}
            </ul>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
