<script lang="ts">
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import PostureScore from '$lib/components/PostureScore.svelte';
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

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    
    <!-- Colonne principale (Score et Alertes/Incidents) -->
    <div class="lg:col-span-2 space-y-6">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <PostureScore />
        
        <!-- Carte Alertes/Résumé -->
        <div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
          <h2 class="text-xl font-bold mb-4 text-slate-100">Alertes</h2>
          <div class="space-y-3">
            <div class="border-l-4 border-red-500 bg-red-900/20 p-3 rounded-r-md">
              <p class="font-semibold text-red-300">Incident ouvert</p>
              <p class="text-sm text-slate-400">Fuite d'email détectée avec une sévérité élevée.</p>
            </div>
            <div class="border-l-4 border-orange-500 bg-orange-900/20 p-3 rounded-r-md">
              <p class="font-semibold text-orange-300">Exposition élevée</p>
              <p class="text-sm text-slate-400">Email et mot de passe haché exposés dans Collection #1.</p>
            </div>
            <div class="border-l-4 border-yellow-500 bg-yellow-900/20 p-3 rounded-r-md">
              <p class="font-semibold text-yellow-300">Demande RGPD en cours</p>
              <p class="text-sm text-slate-400">Demande d'effacement envoyée à Réseau Social X.</p>
            </div>
          </div>
        </div>
      </div>

      <IncidentList />
      <ExposureList />
    </div>

    <!-- Colonne latérale (Chronologie) -->
    <div class="lg:col-span-1 space-y-6">
      <Timeline />
      
      <!-- Carte Actions rapides (placeholder) -->
      <div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
        <h2 class="text-xl font-bold mb-4 text-slate-100">Actions rapides</h2>
        <div class="space-y-2">
          <button class="w-full text-left px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-md transition-colors">
            Ajouter une identité
          </button>
          <button class="w-full text-left px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-md transition-colors">
            Déclarer un incident
          </button>
          <button class="w-full text-left px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-md transition-colors">
            Nouvelle demande RGPD
          </button>
        </div>
      </div>
    </div>

  </div>
</section>
