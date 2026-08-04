<script lang="ts">
	import { onMount } from 'svelte';
	import { listFolders, createFolder, deleteFolder, type Folder } from '$lib/api';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import StatePanel from '$lib/components/StatePanel.svelte';
	let folders=$state<Folder[]>([]),loading=$state(true),creating=$state(false),deleteLoading=$state(false);
	let error=$state<string|null>(null),createError=$state<string|null>(null),deleteError=$state<string|null>(null),deleteId=$state<string|null>(null);
	let newName=$state(''),newContext=$state('');
	onMount(refresh); async function refresh(){try{folders=await listFolders()}catch(e){error=String(e)}finally{loading=false}}
	async function handleCreateFolder(e:SubmitEvent){e.preventDefault();if(!newName.trim()){createError='Le nom du dossier est requis.';return}creating=true;createError=null;try{await createFolder(newName.trim(),newContext.trim());newName='';newContext='';await refresh()}catch(e){createError=String(e)}finally{creating=false}}
	async function handleDeleteFolder(folder:Folder){if(!confirm(`Supprimer le dossier « ${folder.name} » ?`))return;deleteId=folder.id;deleteLoading=true;deleteError=null;try{await deleteFolder(folder.id);folders=folders.filter(item=>item.id!==folder.id)}catch(e){deleteError=String(e)}finally{deleteLoading=false;deleteId=null}}
</script>

<section class="wf-view folders-view">
	<GuideHeader title="Dossiers" question="Comment organiser les identités et observations d’une même recherche ?" intro="Un dossier rassemble le contexte, les identités, les expositions et les décisions liées à une investigation." />
	<div class="folder-layout">
		<form class="glass-card create-panel" onsubmit={handleCreateFolder}>
			<div><span class="eyebrow">Nouveau dossier</span><h2>Définir le contexte</h2><p>Utilisez un nom reconnaissable et décrivez brièvement l’objectif.</p></div>
			<label>Nom du dossier<input bind:value={newName} placeholder="Ex. Présence numérique personnelle" required /></label>
			<label>Contexte<textarea bind:value={newContext} rows="5" placeholder="Périmètre, objectif ou précautions particulières…"></textarea></label>
			{#if createError}<StatePanel compact tone="danger" title="Création impossible" message={createError}/>{/if}
			<button class="wf-btn primary" disabled={creating}>{creating?'Création…':'Créer le dossier'}</button>
		</form>
		<div class="glass-card list-panel">
			<div class="panel-head"><div><span class="eyebrow">Espace de travail</span><h2>Mes dossiers</h2></div><span class="count">{folders.length}</span></div>
			{#if loading}<StatePanel tone="info" title="Chargement des dossiers" message="Lecture de la base locale…"/>
			{:else if error}<StatePanel tone="danger" title="Dossiers indisponibles" message={error}/>
			{:else if folders.length===0}<StatePanel title="Aucun dossier" message="Créez votre premier dossier pour commencer à organiser une investigation."/>
			{:else}<ul class="folder-list">{#each folders as folder(folder.id)}<li><a class="folder-main" href={`/dossiers/${folder.id}`}><span class="folder-icon" aria-hidden="true">⌁</span><span><strong>{folder.name}</strong><small>{folder.context||'Aucun contexte renseigné'}</small></span><span class="arrow">→</span></a><button class="delete-btn" aria-label={`Supprimer ${folder.name}`} disabled={deleteLoading&&deleteId===folder.id} onclick={()=>handleDeleteFolder(folder)}>{deleteLoading&&deleteId===folder.id?'…':'×'}</button></li>{/each}</ul>{/if}
			{#if deleteError}<StatePanel compact tone="danger" title="Suppression impossible" message={deleteError}/>{/if}
		</div>
	</div>
</section>

<style>
	.folders-view{max-width:1280px}.folder-layout{display:grid;grid-template-columns:minmax(280px,.72fr) minmax(0,1.28fr);gap:1rem;align-items:start}.create-panel{display:grid;gap:1rem}.eyebrow{font-size:.65rem;font-weight:750;letter-spacing:.12em;text-transform:uppercase;color:var(--ui-link)}.create-panel h2,.panel-head h2{margin:.25rem 0 0}.create-panel p{margin:.35rem 0 0;color:var(--ui-text-secondary);font-size:.8rem}.create-panel label{display:grid;gap:.4rem;color:var(--ui-text-secondary);font-size:.75rem;font-weight:650}.create-panel input,.create-panel textarea{width:100%;padding:.7rem .75rem;border:1px solid var(--ui-border-default);border-radius:var(--radius-sm);background:rgba(0,0,0,.24);color:var(--ui-text-primary);resize:vertical}.create-panel input:focus,.create-panel textarea:focus{border-color:var(--ui-link);outline:none;box-shadow:0 0 0 3px var(--ui-focus-ring)}.panel-head{display:flex;align-items:center;justify-content:space-between;margin-bottom:1rem}.count{display:grid;place-items:center;min-width:28px;height:28px;border:1px solid var(--ui-border-default);border-radius:50%;color:var(--ui-text-secondary);font-size:.72rem}.folder-list{list-style:none;margin:0;padding:0;display:grid;gap:.55rem}.folder-list li{display:flex;align-items:center;border:1px solid var(--ui-border-default);border-radius:var(--radius-md);background:rgba(0,0,0,.18);overflow:hidden;transition:border-color var(--duration-fast),background var(--duration-fast)}.folder-list li:hover{border-color:color-mix(in srgb,var(--ui-link) 34%,var(--ui-border-default));background:rgba(255,255,255,.035)}.folder-main{display:grid;grid-template-columns:auto 1fr auto;align-items:center;gap:.75rem;min-width:0;flex:1;padding:.8rem;text-decoration:none}.folder-icon{display:grid;place-items:center;width:34px;height:34px;border:1px solid color-mix(in srgb,var(--ui-link) 30%,var(--ui-border-default));border-radius:9px;color:var(--ui-link);background:var(--ui-link-soft)}.folder-main strong,.folder-main small{display:block}.folder-main strong{color:var(--ui-text-primary);font-size:.86rem}.folder-main small{margin-top:.18rem;color:var(--ui-text-secondary);font-size:.74rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.arrow{color:var(--ui-link)}.delete-btn{align-self:stretch;width:42px;border:0;border-left:1px solid var(--ui-border-subtle);background:transparent;color:var(--ui-text-tertiary);font-size:1rem}.delete-btn:hover{background:rgba(240,82,97,.1);color:var(--ui-danger)}@media(max-width:850px){.folder-layout{grid-template-columns:1fr}}
</style>
