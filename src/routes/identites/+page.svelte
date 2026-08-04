<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import StatePanel from '$lib/components/StatePanel.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { t } from '$lib/i18n';
  import {
    listIdentities, listFolders, listExposures, createIdentity, updateIdentity, deleteIdentity,
    type Identity, type IdentityInput, type IdentityValueInput, type IdentityValueKind,
    type Folder, type Exposure
  } from '$lib/api';

  const kindLabels: Record<IdentityValueKind, string> = {
    prenom: 'Prénom', nom: 'Nom', pseudo: 'Pseudo', email: 'E-mail',
    telephone: 'Téléphone', adresse: 'Adresse', domaine: 'Domaine (hérité)', url: 'URL (héritée)'
  };
  const editableKinds: IdentityValueKind[] = ['prenom', 'nom', 'pseudo', 'email', 'telephone', 'adresse'];

  let identities = $state<Identity[]>([]);
  let folders = $state<Folder[]>([]);
  let exposures = $state<Exposure[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let mode = $state<'view' | 'edit' | 'create'>('view');
  let formData = $state<IdentityInput>(emptyForm());

  function emptyValue(kind: IdentityValueKind = 'prenom'): IdentityValueInput {
    return { kind, value: '', label: null, status: 'active', address_line1: null, address_line2: null, city: null, postal_code: null, country: null };
  }

  function emptyForm(): IdentityInput {
    return { label: '', status: 'active', folder_id: null, notes: null, values: [emptyValue()] };
  }

  function formFromIdentity(identity: Identity): IdentityInput {
    return {
      label: identity.label,
      status: identity.status,
      folder_id: identity.folder_id,
      notes: identity.notes,
      values: identity.values.map((value) => ({
        id: value.id, kind: value.kind, value: value.value, label: value.label, status: value.status,
        address_line1: value.address_line1, address_line2: value.address_line2, city: value.city,
        postal_code: value.postal_code, country: value.country
      }))
    };
  }

  onMount(async () => {
    try {
      [identities, folders, exposures] = await Promise.all([listIdentities(), listFolders(), listExposures()]);
    } catch (cause) {
      error = String(cause);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? identities[0]?.id ?? null);
  const selected = $derived(identities.find((identity) => identity.id === selectedId));

  function getFolderName(folderId: string | null): string {
    return folderId ? folders.find((folder) => folder.id === folderId)?.name ?? 'Dossier inconnu' : 'Aucun dossier';
  }

  function exposuresForIdentity(identity: Identity | undefined): Exposure[] {
    return identity?.folder_id ? exposures.filter((exposure) => exposure.folder_id === identity.folder_id) : [];
  }

  function startCreate() {
    error = null;
    mode = 'create';
    formData = emptyForm();
    formData.folder_id = folders[0]?.id ?? null;
  }

  function startEdit() {
    if (!selected) return;
    error = null;
    mode = 'edit';
    formData = formFromIdentity(selected);
  }

  function cancelEdit() {
    mode = 'view';
    error = null;
  }

  function addValue(kind: IdentityValueKind = 'prenom') {
    formData.values = [...formData.values, emptyValue(kind)];
  }

  function removeValue(index: number) {
    if (formData.values.length === 1) return;
    formData.values = formData.values.filter((_, current) => current !== index);
  }

  async function saveIdentity() {
    saving = true;
    error = null;
    try {
      const payload: IdentityInput = {
        ...formData,
        label: formData.label.trim(),
        notes: formData.notes?.trim() || null,
        folder_id: formData.folder_id || null,
        values: formData.values.map((value) => ({
          ...value,
          value: value.value.trim(),
          label: value.label?.trim() || null,
          address_line1: value.address_line1?.trim() || null,
          address_line2: value.address_line2?.trim() || null,
          city: value.city?.trim() || null,
          postal_code: value.postal_code?.trim() || null,
          country: value.country?.trim() || null
        }))
      };
	      if (mode === 'create') {
        const created = await createIdentity(payload);
        identities = [...identities, created];
        mode = 'view';
        window.dispatchEvent(new CustomEvent('mantis-identities-changed', { detail: { id: created.id, action: 'created' } }));
        await goto(`/identites?id=${created.id}`, { replaceState: true, noScroll: true });
      } else if (mode === 'edit' && selected) {
        const updated = await updateIdentity(selected.id, payload);
        identities = identities.map((identity) => identity.id === updated.id ? updated : identity);
        mode = 'view';
        window.dispatchEvent(new CustomEvent('mantis-identities-changed', { detail: { id: updated.id, action: 'updated' } }));
      }
    } catch (cause) {
      error = String(cause);
    } finally {
      saving = false;
    }
  }

  async function removeIdentity(id: string) {
    if (!confirm('Supprimer cette identité et ses données saisies ? Les relations existantes peuvent empêcher la suppression.')) return;
    try {
      await deleteIdentity(id);
      identities = identities.filter((identity) => identity.id !== id);
      window.dispatchEvent(new CustomEvent('mantis-identities-changed', { detail: { id, action: 'deleted' } }));
      await goto('/identites', { replaceState: true, noScroll: true });
    } catch (cause) {
      error = String(cause);
    }
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Identités"
    question="Quelles informations souhaitez-vous utiliser comme points de départ ?"
    intro="Vous contrôlez entièrement ces informations. Les éléments observés restent séparés et ne sont jamais ajoutés sans votre accord."
  />

  <ol class="identity-journey" aria-label="Parcours d’une identité">
    <li class="active"><span>01</span><div><strong>{t('Renseigner')}</strong><small>{t('Vos propres identifiants')}</small></div></li>
    <li><span>02</span><div><strong>{t('Scanner')}</strong><small>{t('Les sources compatibles')}</small></div></li>
    <li><span>03</span><div><strong>{t('Examiner')}</strong><small>{t('Sans attribution automatique')}</small></div></li>
  </ol>

  {#if error}<StatePanel tone="danger" title="Action impossible" message={error} live="assertive" />{/if}

  {#if loading}
    <StatePanel tone="info" title="Chargement des identités" message="Lecture des informations conservées sur cet appareil…" />
  {:else}
    <div class="split-layout">
      <aside class="glass-card list-panel">
        <div class="list-header"><div><p class="eyebrow">Points de départ</p><h2>Mes identités</h2><p>{identities.length} identité{identities.length > 1 ? 's' : ''}</p></div><button class="wf-btn primary small" onclick={startCreate}>+ Nouvelle</button></div>
        <ul class="item-list">
          {#each identities as identity (identity.id)}
            <li>
              <a class="list-item" class:active={selectedId === identity.id && mode === 'view'} class:inactive={identity.status === 'inactive'} href={`/identites?id=${identity.id}`}>
                <span class="item-title"><span class="status-dot"></span>{identity.label}</span>
                <span class="item-desc">{identity.values.length} {t('données actives')} · {identity.status === 'active' ? 'Active' : 'Inactive'}</span>
              </a>
            </li>
          {:else}
            <li><EmptyState title="Aucune identité" message="Ajoutez uniquement les informations que vous souhaitez utiliser comme points de départ." /></li>
          {/each}
        </ul>
      </aside>

      <div class="glass-card detail-panel">
        {#if mode === 'view' && selected}
          <div class="detail-header">
            <div><p class="eyebrow">Identité suivie</p><h3>{selected.label}</h3><p class="muted">{getFolderName(selected.folder_id)}</p></div>
            <StatusBadge tone={selected.status === 'active' ? 'success' : 'neutral'} dot label={selected.status === 'active' ? 'Active' : 'Inactive'} />
          </div>

          <div class="separation-note"><strong>Renseigné par vous</strong><span>Ces valeurs servent de points de départ. Elles ne proviennent pas des résultats OSINT.</span></div>
          <div class="values-grid">
            {#each selected.values as value (value.id)}
              <article class:inactive-value={value.status === 'inactive'} class="value-card">
                <div class="value-head"><span class="kind">{kindLabels[value.kind]}</span><span>{value.status === 'active' ? 'Utilisée' : 'En pause'}</span></div>
                <strong>{value.value}</strong>
                {#if value.label}<small>{value.label}</small>{/if}
              </article>
            {/each}
          </div>

          {#if selected.notes}<div class="notes"><span>Notes</span><p>{selected.notes}</p></div>{/if}

          <div class="observed-zone"><div><p class="eyebrow">Étape suivante</p><strong>Scanner les traces publiques</strong><p>Les résultats resteront séparés et ne modifieront jamais les données ci-dessus.</p></div><a class="wf-btn primary" href={`/veille?identity=${selected.id}`}>Préparer le scan →</a></div>

          <div class="actions-section">
            <h4>{t('Expositions du même dossier')} ({exposuresForIdentity(selected).length})</h4>
            {#if exposuresForIdentity(selected).length}
              <ul class="sub-list">{#each exposuresForIdentity(selected) as exposure (exposure.id)}<li><a href={`/expositions?id=${exposure.id}`}>{exposure.title}</a></li>{/each}</ul>
            {:else}<p class="muted">{t('Aucune exposition dans ce dossier.')}</p>{/if}
          </div>
          <div class="action-buttons"><button class="wf-btn primary" onclick={startEdit}>Modifier</button><button class="wf-btn danger" onclick={() => removeIdentity(selected.id)}>Supprimer</button></div>

        {:else if mode === 'edit' || mode === 'create'}
          <div class="detail-header"><div><p class="eyebrow">Données saisies par l’utilisateur</p><h3>{mode === 'create' ? 'Nouvelle identité' : `Modifier ${selected?.label ?? ''}`}</h3></div></div>
          <form class="identity-form" onsubmit={(event) => { event.preventDefault(); saveIdentity(); }}>
            <div class="identity-fields">
              <label>Nom d’affichage<input bind:value={formData.label} required placeholder="Ex. Mon identité principale" /></label>
              <label>Statut<select bind:value={formData.status}><option value="active">Active — incluse dans les routines</option><option value="inactive">Inactive — exclue des routines</option></select></label>
              <label>Dossier<select bind:value={formData.folder_id}><option value={null}>Aucun dossier</option>{#each folders as folder (folder.id)}<option value={folder.id}>{folder.name}</option>{/each}</select></label>
            </div>

            <div class="values-editor">
              <div class="editor-title"><div><h4>Données de recherche</h4><p>Ajoutez plusieurs noms, pseudos, e-mails, téléphones ou adresses.</p></div><button type="button" class="wf-btn small" onclick={() => addValue()}>+ Ajouter</button></div>
              {#each formData.values as value, index (value.id ?? index)}
                <fieldset class="value-editor">
                  <legend>Donnée {index + 1}</legend>
                  <div class="value-row">
                    <label>Type<select bind:value={value.kind}>{#each editableKinds as kind}<option value={kind}>{kindLabels[kind]}</option>{/each}{#if !editableKinds.includes(value.kind)}<option value={value.kind}>{kindLabels[value.kind]}</option>{/if}</select></label>
                    <label class="grow">Valeur<input bind:value={value.value} required placeholder={value.kind === 'email' ? 'nom@exemple.fr' : 'Valeur connue'} /></label>
                    <label>État<select bind:value={value.status}><option value="active">Active</option><option value="inactive">Inactive</option></select></label>
                    <button type="button" class="icon-btn" aria-label="Retirer cette donnée" disabled={formData.values.length === 1} onclick={() => removeValue(index)}>×</button>
                  </div>
                  <label class="optional">Libellé facultatif<input bind:value={value.label} placeholder="Ex. professionnel, ancien…" /></label>
                  {#if value.kind === 'adresse'}
                    <div class="address-grid">
                      <label class="wide">Adresse<input bind:value={value.address_line1} /></label><label class="wide">Complément<input bind:value={value.address_line2} /></label>
                      <label>Ville<input bind:value={value.city} /></label><label>Code postal<input bind:value={value.postal_code} /></label><label>Pays<input bind:value={value.country} /></label>
                    </div>
                  {/if}
                </fieldset>
              {/each}
            </div>
            <label>Notes<textarea bind:value={formData.notes} placeholder="Contexte personnel, sans mot de passe ni secret"></textarea></label>
            <div class="action-buttons"><button class="wf-btn primary" type="submit" disabled={saving}>{saving ? 'Enregistrement…' : 'Enregistrer'}</button><button class="wf-btn" type="button" onclick={cancelEdit}>Annuler</button></div>
          </form>
        {:else}
          <div class="detail-empty"><EmptyState title="Créez une identité" message="Regroupez les informations connues d’une même personne avant de lancer Scanner." /><button class="wf-btn primary" onclick={startCreate}>Nouvelle identité</button></div>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .muted,.list-header p,.editor-title p,.observed-zone p { color:var(--mantis-text-muted); font-size:.82rem; margin:.25rem 0 0; }
	.identity-journey { list-style:none; display:grid; grid-template-columns:repeat(3,1fr); gap:1px; margin:0; padding:0; overflow:hidden; border:1px solid var(--ui-border-default); border-radius:var(--radius-md); background:var(--ui-border-default); }.identity-journey li { display:flex; align-items:center; gap:.65rem; min-height:58px; padding:.65rem .8rem; background:var(--ui-surface-1); }.identity-journey li.active { box-shadow:inset 3px 0 0 var(--ui-accent); background:color-mix(in srgb,var(--ui-accent) 6%,var(--ui-surface-1)); }.identity-journey>li>span { color:var(--ui-text-tertiary); font-family:var(--font-meta); font-size:.68rem; }.identity-journey strong,.identity-journey small { display:block; }.identity-journey strong { font-size:.78rem; }.identity-journey small { margin-top:.1rem; color:var(--ui-text-tertiary); font-size:.68rem; }
  .split-layout { display:grid; grid-template-columns:minmax(260px, .8fr) minmax(0,2.2fr); gap:1rem; align-items:start; }
  .list-panel,.detail-panel { min-width:0; } .list-header,.detail-header,.editor-title,.observed-zone,.value-head,.action-buttons { display:flex; justify-content:space-between; align-items:center; gap:1rem; }
  h2,h3,h4 { margin:0; } .list-header h2 { font-size:1rem; }
  .item-list,.sub-list { list-style:none; padding:0; margin:1rem 0 0; display:grid; gap:.5rem; }
  .list-item { display:grid; gap:.35rem; padding:.8rem; border:1px solid var(--mantis-border); border-radius:10px; color:inherit; text-decoration:none; background:rgba(0,0,0,.2); }
  .list-item:hover { border-color:var(--ui-border-strong); background:var(--ui-surface-2); }.list-item.active { border-color:color-mix(in srgb,var(--ui-link) 30%,var(--ui-border-default)); background:color-mix(in srgb,var(--ui-link) 7%,var(--ui-surface-1)); box-shadow:inset 2px 0 0 var(--ui-link); }
  .list-item.inactive { opacity:.65; } .item-title { font-weight:700; display:flex; align-items:center; gap:.5rem; } .item-desc { color:var(--mantis-text-muted); font-size:.76rem; }
  .status-dot { width:7px; height:7px; border-radius:50%; background:var(--ui-success); } .inactive .status-dot { background:var(--mantis-text-muted); }
  .eyebrow { margin:0 0 .35rem; color:var(--mantis-accent); font-size:.68rem; letter-spacing:.12em; text-transform:uppercase; font-weight:700; }
  .kind { font-size:.68rem; letter-spacing:.06em; text-transform:uppercase; border:1px solid var(--mantis-border); border-radius:999px; padding:.25rem .55rem; color:var(--mantis-text-muted); }
  .separation-note { display:grid; gap:.2rem; padding:.8rem 1rem; margin:1.2rem 0; border-left:3px solid var(--mantis-accent); background:color-mix(in srgb,var(--mantis-accent) 7%,transparent); border-radius:0 9px 9px 0; } .separation-note span { color:var(--mantis-text-muted); font-size:.8rem; }
  .values-grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(210px,1fr)); gap:.7rem; }
  .value-card { display:grid; gap:.65rem; padding:.9rem; border:1px solid var(--mantis-border); border-radius:11px; background:rgba(0,0,0,.22); } .value-card strong { overflow-wrap:anywhere; } .value-card small,.value-head span:last-child { color:var(--mantis-text-muted); font-size:.72rem; } .inactive-value { opacity:.58; }
  .notes,.actions-section,.observed-zone { margin-top:1.25rem; padding-top:1rem; border-top:1px solid var(--mantis-border); } .notes span { color:var(--mantis-text-muted); font-size:.72rem; text-transform:uppercase; } .notes p { white-space:pre-wrap; }
  .observed-zone { padding:1rem; border:1px solid var(--mantis-border); border-radius:11px; background:rgba(255,255,255,.025); } .sub-list a { color:var(--mantis-text); }
  .action-buttons { justify-content:flex-start; margin-top:1.25rem; flex-wrap:wrap; } .wf-btn.small { padding:.38rem .7rem; font-size:.78rem; } .wf-btn.danger { color:#ffd8dc; border-color:color-mix(in srgb,var(--mantis-danger) 45%,transparent); }
  .identity-form,.values-editor { display:grid; gap:1rem; } .identity-fields { display:grid; grid-template-columns:2fr 1fr 1fr; gap:.8rem; }
  label { display:grid; gap:.4rem; color:var(--mantis-text-muted); font-size:.72rem; text-transform:uppercase; letter-spacing:.06em; }
  input,select,textarea { width:100%; box-sizing:border-box; padding:.65rem .75rem; border:1px solid var(--mantis-border-strong); border-radius:8px; background:rgba(0,0,0,.28); color:var(--mantis-text); font:inherit; text-transform:none; letter-spacing:normal; } textarea { min-height:80px; resize:vertical; }
  input:focus,select:focus,textarea:focus { outline:none; border-color:var(--mantis-accent); }
  .values-editor { margin-top:.25rem; padding:1rem; border:1px solid var(--mantis-border); border-radius:12px; background:rgba(0,0,0,.14); }
  .value-editor { margin:0; padding:1rem; border:1px solid var(--mantis-border); border-radius:10px; } .value-editor legend { padding:0 .4rem; color:var(--mantis-text-muted); font-size:.75rem; }
  .value-row { display:flex; gap:.7rem; align-items:end; } .value-row .grow { flex:1; } .icon-btn { width:38px; height:38px; border:1px solid var(--mantis-border); border-radius:8px; background:transparent; color:var(--mantis-text); font-size:1.25rem; cursor:pointer; } .icon-btn:disabled { opacity:.3; cursor:not-allowed; }
  .optional { margin-top:.7rem; max-width:360px; } .address-grid { margin-top:.8rem; display:grid; grid-template-columns:repeat(3,1fr); gap:.7rem; } .address-grid .wide { grid-column:span 3; }
  .detail-empty { display:grid; justify-items:center; text-align:center; padding:1rem; } .detail-empty .wf-btn { margin-top:1rem; }
  @media (max-width:1000px) { .split-layout { grid-template-columns:1fr; } .item-list { grid-template-columns:repeat(auto-fit,minmax(210px,1fr)); } }
  @media (max-width:700px) { .identity-journey { grid-template-columns:1fr; }.identity-fields,.address-grid { grid-template-columns:1fr; } .address-grid .wide { grid-column:auto; } .value-row { align-items:stretch; flex-direction:column; } .icon-btn { align-self:end; } .observed-zone { align-items:flex-start; flex-direction:column; } }

	.identity-journey { border-color:var(--ui-border-subtle); background:var(--ui-border-subtle); }
	.identity-journey li { background:var(--ui-material-solid); }
	.identity-journey li.active { background:color-mix(in srgb,var(--ui-accent) 7%,var(--ui-material-solid)); box-shadow:inset 2px 0 0 var(--ui-accent); }
	.split-layout .list-panel,
	.split-layout .detail-panel { background:var(--ui-material-panel); }
	.identity-journey + .split-layout .list-item { background:var(--ui-material-solid); }
	.identity-journey + .split-layout .list-item.active { border-color:color-mix(in srgb,var(--ui-accent) 32%,var(--ui-border-default)); background:color-mix(in srgb,var(--ui-accent) 7%,var(--ui-material-solid)); box-shadow:inset 2px 0 0 var(--ui-accent); }
	.identity-journey + .split-layout .value-card,
	.identity-journey + .split-layout .values-editor,
	.identity-journey + .split-layout .value-editor { background:var(--ui-material-solid); }
	.identity-journey + .split-layout input,
	.identity-journey + .split-layout select,
	.identity-journey + .split-layout textarea { background:var(--ui-material-solid); }
</style>
