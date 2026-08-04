<script lang="ts">
  import { onMount } from 'svelte';
  import { createAction, getIncident, type Incident } from '$lib/api';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import StatePanel from '$lib/components/StatePanel.svelte';
	import RemediationJourney from '$lib/components/RemediationJourney.svelte';

  let incidentId = $state<string | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);
  let incidentLoading = $state(true);
  let incidentError = $state<string | null>(null);
  let incident = $state<Incident | null>(null);

  // Form fields
  let title = $state('');
  let priority = $state('prio_002'); // moyenne by default
  let difficulty = $state('diff_002'); // moyenne by default
  let deadline = $state('');
  let guidanceLines = $state('');
  let proofExpected = $state('');

  // Options for selects
  const priorityOptions = [
    { value: 'prio_001', label: 'Basse' },
    { value: 'prio_002', label: 'Moyenne' },
    { value: 'prio_003', label: 'Haute' },
    { value: 'prio_004', label: 'Critique' }
  ];
  const difficultyOptions = [
    { value: 'diff_001', label: 'Facile' },
    { value: 'diff_002', label: 'Moyenne' },
    { value: 'diff_003', label: 'Difficile' }
  ];

  // Initialize from route params
  onMount(() => {
    const unsub = page.subscribe($page => {
      incidentId = $page.url.searchParams.get('incidentId');
      if (incidentId) {
        loadIncident(incidentId);
      } else {
        // Reset form when no incidentId
        title = '';
        priority = 'prio_002';
        difficulty = 'diff_002';
        deadline = '';
        guidanceLines = '';
        proofExpected = '';
        incident = null;
        incidentLoading = false;
        incidentError = null;
      }
    });
    return unsub;
  });

  async function loadIncident(id: string) {
    try {
      incident = await getIncident(id);
      // Pre-fill form fields
      title = incident.title;
      guidanceLines = incident.next_step;
      // Leave priority, difficulty, deadline for user to choose (could be set based on incident but keep flexible)
    } catch (e) {
      incidentError = String(e);
    } finally {
      incidentLoading = false;
    }
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!title.trim()) {
      error = 'Le titre est requis';
      return;
    }
    if (!deadline) {
      error = 'La date d\'échéance est requise';
      return;
    }

    loading = true;
    error = null;
    success = null;

    try {
      // Convert guidance lines to JSON array
      const guidanceArray = guidanceLines
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.length > 0);
      const guidanceJson = JSON.stringify(guidanceArray);

      const action = await createAction(
        title.trim(),
        priority,
        difficulty,
        deadline,
        guidanceJson,
        proofExpected.trim(),
        null, // folder_id (optional)
        incidentId
      );

      success = `Action créée avec succès (ID: ${action.id})`;
      // Reset form
      title = '';
      priority = 'prio_002';
      difficulty = 'diff_002';
      deadline = '';
      guidanceLines = '';
      proofExpected = '';
      incident = null;
      // Optionally redirect to actions list after a short delay
      setTimeout(() => {
        goto('/actions');
      }, 1500);
    } catch (err) {
      error = `Erreur : ${err}`;
    } finally {
      loading = false;
    }
  }
</script>

<section class="wf-view action-create-view">
	<GuideHeader title="Nouvelle action" question="Quelle étape concrète réduira ce risque ?" intro="Définissez une action vérifiable, sa priorité, son échéance et la preuve attendue." />
	<RemediationJourney current="action" />
<div class="glass-card action-form-card">
  {#if incidentId}
    {#if incidentLoading}
      <StatePanel compact tone="info" title="Chargement de l’incident" />
    {:else if incidentError}
      <StatePanel compact tone="danger" title="Incident indisponible" message={incidentError} />
    {:else}
      <StatePanel compact tone="info" title="Incident associé" message={incident?.title ?? incidentId} />
    {/if}
  {/if}

  {#if error}
    <StatePanel compact tone="danger" title="Action non créée" message={error} />
  {/if}
  {#if success}
    <StatePanel compact tone="success" title="Action créée" message={success} />
  {/if}

  <form onsubmit={handleSubmit}>
    <div class="form-group">
      <label for="title">Titre *</label>
      <input
        id="title"
        type="text"
        bind:value={title}
        placeholder="Ex: Changer le mot de passe du compte email"
        required
      />
    </div>

    <div class="form-group">
      <label for="priority">Priorité</label>
      <select id="priority" bind:value={priority}>
        {#each priorityOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>

    <div class="form-group">
      <label for="difficulty">Difficulté</label>
      <select id="difficulty" bind:value={difficulty}>
        {#each difficultyOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>

    <div class="form-group">
      <label for="deadline">Date d'échéance *</label>
      <input id="deadline" type="date" bind:value={deadline} required />
    </div>

    <div class="form-group">
      <label for="guidance">Étapes (une ligne par étape)</label>
      <textarea
        id="guidance"
        bind:value={guidanceLines}
        rows="4"
        placeholder="Étape 1\nÉtape 2\nÉtape 3"
      ></textarea>
    </div>

    <div class="form-group">
      <label for="proof">Preuve attendue</label>
      <input
        id="proof"
        type="text"
        bind:value={proofExpected}
        placeholder="Ex: Capture d'écran montrant le mot de passe modifié"
      />
    </div>

    <div class="form-actions">
      <button type="submit" class="wf-btn primary" disabled={loading}>
        {#if loading}Création...{:else}Créer l'action{/if}
      </button>
      <a href="/actions" class="wf-btn">Annuler</a>
    </div>
  </form>
</div>
</section>

<style>
	.action-create-view { max-width:850px; }
	.action-form-card { padding:1.2rem; }

  .form-group {
    margin-bottom: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.25rem;
    font-weight: 600;
  }

  input[type="text"],
  input[type="date"],
  textarea,
  select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--mantis-border);
    border-radius: 4px;
    background: var(--mantis-bg);
    color: var(--mantis-text);
    font-family: inherit;
    font-size: 0.9rem;
  }

  input[type="text"]:focus,
  input[type="date"]:focus,
  textarea:focus,
  select:focus {
    outline: none;
    border-color: var(--mantis-accent);
  }

  textarea {
    resize: vertical;
  }

  .form-actions {
    margin-top: 1.5rem;
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

</style>
