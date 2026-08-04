<script lang="ts">
  import { onMount } from 'svelte';
  import { getIdentity, type Identity } from '$lib/api';
  import { listExposures, type Exposure } from '$lib/api';
  import { listIncidents, type Incident } from '$lib/api';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  let identityId = $state<string | null>(null);
  let identity = $state<Identity | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let exposures: Exposure[] = [];
  let incidents: Incident[] = [];
  let dataLoading = true;
  let dataError: string | null = null;

  // Helper to get label for identity kind
  const identityKindLabel: Record<string, string> = {
    prenom: 'Prénom',
    nom: 'Nom',
    email: 'E-mail',
    telephone: 'Téléphone',
    pseudo: 'Pseudo',
    domaine: 'Domaine',
    url: 'URL / profil',
    adresse: 'Adresse'
  };

  // Helper to get exposures for an identity (same folder)
  function exposuresForIdentity(identity: Identity | undefined): Exposure[] {
    if (!identity || !identity.folder_id) return [];
    return exposures.filter(e => e.folder_id === identity.folder_id);
  }

  // Helper to get incidents for an identity (same folder)
  function incidentsForIdentity(identity: Identity | undefined): Incident[] {
    if (!identity || !identity.folder_id) return [];
    return incidents.filter(i => i.folder_id === identity.folder_id);
  }

  onMount(() => {
    const unsub = page.subscribe($page => {
      identityId = $page.params.id ?? null;
      if (identityId) {
        loadIdentityData();
      } else {
        identity = null;
        loading = false;
        error = null;
        exposures = [];
        incidents = [];
        dataLoading = false;
        dataError = null;
      }
    });
    return unsub;
  });

  async function loadIdentityData() {
    try {
      // Load identity
      identity = await getIdentity(identityId!);
      // Load related exposures and incidents
      const [exps, incs] = await Promise.all([
        listExposures(),
        listIncidents()
      ]);
      exposures = exps;
      incidents = incs;
      dataError = null;
    } catch (e) {
      error = String(e);
      dataError = String(e);
    } finally {
      loading = false;
      dataLoading = false;
    }
  }
</script>

{#if loading}
  <p class="muted">Chargement de l'identité...</p>
{:else if error}
  <p class="error">Erreur: {error}</p>
{:else if identity}
  <div class="glass-card">
    <div class="identity-header">
      <h1>{identity.label}</h1>
      <p class="identity-sub">{identity.status === 'active' ? 'Identité active' : 'Identité inactive'} · {identity.values.length} donnée{identity.values.length > 1 ? 's' : ''} saisie{identity.values.length > 1 ? 's' : ''}</p>
    </div>

    <div class="identity-details">
      <p><strong>Renseigné par vous :</strong></p>
      <ul class="identity-values">
        {#each identity.values as value (value.id)}
          <li class:inactive={value.status === 'inactive'}>
            <span>{identityKindLabel[value.kind] ?? value.kind}</span>
            <strong>{value.value}</strong>
            <small>{value.status === 'active' ? 'Utilisée' : 'En pause'}</small>
          </li>
        {/each}
      </ul>
      {#if identity.notes}
        <p><strong>Notes :</strong> {identity.notes}</p>
      {/if}
      {#if identity.folder_id}
        <p><strong>Dossier :</strong> {identity.folder_id}</p>
      {/if}
    </div>

    {#if exposuresForIdentity(identity).length > 0}
      <section class="section">
        <h2>Expositions liées ({exposuresForIdentity(identity).length})</h2>
        <ul class="list">
          {#each exposuresForIdentity(identity) as exp (exp.id)}
            <li class="list-item">
              <div class="list-content">
                <strong>{exp.title}</strong> (<span class="severity">{exp.severity}</span>)
                <br>
                <span class="what">{exp.what}</span>
              </div>
              <div class="list-actions">
                <a href={`/expositions/${exp.id}`} class="btn btn-sm">Voir</a>
              </div>
            </li>
          {/each}
        </ul>
      </section>
    {:else}
      <p class="muted">Aucune exposition liée à cette identité.</p>
    {/if}

    {#if incidentsForIdentity(identity).length > 0}
      <section class="section">
        <h2>Incidents liés ({incidentsForIdentity(identity).length})</h2>
        <ul class="list">
          {#each incidentsForIdentity(identity) as incident (incident.id)}
            <li class="list-item">
              <div class="list-content">
                <strong>{incident.title}</strong> (<span class="severity">{incident.severity}</span>)
                <br>
                <span class="what">{incident.what}</span>
              </div>
              <div class="list-actions">
                <a href={`/incidents/${incident.id}`} class="btn btn-sm">Voir</a>
              </div>
            </li>
          {/each}
        </ul>
      </section>
    {:else}
      <p class="muted">Aucun incident lié à cette identité.</p>
    {/if}
  </div>
{/if}

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }

  .identity-header {
    margin-bottom: 1.5rem;
    text-align: center;
  }

  .identity-header h1 {
    margin: 0 0 0.5rem 0;
    font-size: 2rem;
    font-weight: 600;
  }

  .identity-sub {
    font-size: 1rem;
    color: var(--mantis-text-muted);
  }

  .identity-details {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: var(--mantis-bg-raised);
    border-radius: var(--radius-md);
  }

  .identity-values { list-style:none; padding:0; margin:.75rem 0 1.25rem; display:grid; gap:.5rem; }
  .identity-values li { display:grid; grid-template-columns:110px 1fr auto; gap:.75rem; align-items:center; padding:.7rem .8rem; border:1px solid var(--mantis-border); border-radius:8px; background:rgba(0,0,0,.2); }
  .identity-values li.inactive { opacity:.55; }
  .identity-values span,.identity-values small { color:var(--mantis-text-muted); font-size:.75rem; }
  .identity-values strong { overflow-wrap:anywhere; }

  .identity-details p {
    margin: 0.25rem 0;
  }

  .section {
    margin-top: 1.5rem;
  }

  .section h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
    border-bottom: 1px solid var(--mantis-border);
    padding-bottom: 0.25rem;
  }

  .list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .list-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border: 1px solid var(--mantis-border);
    border-radius: var(--radius-md);
    margin-bottom: 0.5rem;
    background: var(--mantis-bg-raised);
  }

  .list-content {
    flex: 1;
  }

  .list-actions {
    display: flex;
    gap: 0.5rem;
  }

  .severity {
    font-weight: 600;
    text-transform: uppercase;
    font-size: 0.8rem;
  }
  .btn {
    display: inline-block;
    padding: 0.35rem 0.7rem;
    border-radius: 4px;
    text-decoration: none;
    font-weight: 600;
    cursor: pointer;
    border: none;
    font-size: 0.8rem;
    background: var(--mantis-bg);
    color: var(--mantis-text);
    border: 1px solid var(--mantis-border);
    transition: background 0.2s, opacity 0.2s;
  }

  .btn-sm {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
  }

  .btn:hover {
    background: var(--mantis-bg-raised);
    opacity: 0.9;
  }
</style>
