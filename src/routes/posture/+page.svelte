<script lang="ts">
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import PostureScore from '$lib/components/PostureScore.svelte';
  import AlertList from '$lib/components/AlertList.svelte';
  import IncidentList from '$lib/components/IncidentList.svelte';
  import ExposureList from '$lib/components/ExposureList.svelte';
  import Timeline from '$lib/components/Timeline.svelte';
</script>

<svelte:head>
  <title>Centre de posture - MANTIS</title>
</svelte:head>

<section class="wf-view">
  <GuideHeader
    title="Centre de posture"
    question="Où en suis-je, et que dois-je faire en premier ?"
    intro="Ce hub résume votre exposition numérique et vous oriente vers la prochaine action utile — sans jargon technique."
  />

  <div class="dashboard-grid">
    <!-- Colonne principale -->
    <div class="main-col">
      <div class="top-row">
        <PostureScore />
        <AlertList />
      </div>

      <IncidentList />
      <ExposureList />
    </div>

    <!-- Colonne latérale -->
    <div class="side-col">
      <Timeline />
      
      <div class="card quick-actions">
        <h2>Actions rapides</h2>
        <div class="action-buttons">
          <a class="btn" href="/identites">Ajouter une identité</a>
          <a class="btn" href="/incidents">Déclarer un incident</a>
          <a class="btn" href="/dpo">Nouvelle demande RGPD</a>
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .dashboard-grid {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 1.25rem;
  }

  @media (max-width: 1000px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
    }
  }

  .main-col, .side-col {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .top-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }

  @media (max-width: 700px) {
    .top-row {
      grid-template-columns: 1fr;
    }
  }

  .card {
    background: var(--mantis-bg-raised);
    border: 1px solid var(--mantis-border);
    border-radius: 10px;
    padding: 1.25rem;
  }

  .quick-actions h2 {
    margin: 0 0 1rem;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .action-buttons {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .btn {
    display: block;
    width: 100%;
    text-align: left;
    padding: 0.65rem 0.85rem;
    border-radius: 6px;
    border: 1px solid var(--mantis-border);
    background: var(--mantis-bg);
    color: var(--mantis-text);
    font-size: 0.85rem;
    font-weight: 500;
    text-decoration: none;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
  }

  .btn:hover {
    border-color: var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 8%, var(--mantis-bg));
  }
</style>
