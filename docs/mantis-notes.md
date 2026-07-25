# MANTIS POSTURE — Journal technique

## Phase 1 — Socle local-first et cockpit UI

### Étape 1 : Correction du socle Tauri + SvelteKit
- package.json : script `"tauri": "tauri"`, `@sveltejs/adapter-static`.
- vite.config.ts : adapter-static, `strictPort: 5173`, `watch.ignored` excluant `src-tauri/**`.
- src-tauri/tauri.conf.json : `devUrl` 5173, `frontendDist` ../build.

### Étape 2 : UI cockpit sombre de base
- src/app.css : palette cockpit graphite/ardoise avec variables CSS.
- src/app.html : `lang="fr"`, fond anti-flash.
- src/routes/+layout.svelte : structure shell (sidebar + zone de contenu), marque MANTIS POSTURE.
- src/routes/+page.svelte : page d'accueil avec statut du socle.

### Étape 3 : Fenêtre moderne + navigation complète
- src-tauri/tauri.conf.json : fenêtre sans décorations (`decorations: false`), transparente, 1280×840, centrée.
- src-tauri/capabilities/default.json : permissions window (minimize, maximize, close, drag).
- src/lib/Titlebar.svelte : barre de titre custom avec marque, phase, boutons contrôle.
- src/routes/+layout.svelte : intégration Titlebar + navigation cliquable avec 12 liens.
- src/routes/+layout.ts : `prerender = true` pour build statique.
- src/routes/posture/+page.svelte : Centre de posture avec score fictif (72) et alertes mock.
- src/lib/ViewPlaceholder.svelte : composant réutilisable pour vues à venir.
- Routes créées : /posture, /dossiers, /graphe, /identites, /veille, /expositions, /incidents, /actions, /dpo, /guides, /rapports, /parametres.
- src/routes/+page.svelte : redirection vers /posture.

### Étape 4 : Correction build — capabilities/default.json corrompu
- Erreur cargo build-script : `failed to parse JSON: expected value at line 1 column 1`.
- Cause : le fichier contenait du texte de narration au lieu du JSON (fuite des outils d'édition).
- Correction : réécriture via PowerShell `[System.IO.File]::WriteAllText` en UTF-8 sans BOM.
- Vérification : `cargo check --no-default-features` passe (~12 s), toutes les dépendances compilent.
- Règle : pour les fichiers sensibles (JSON Tauri), écrire via terminal et vérifier les premiers octets (0x7B = `{`).

### Étape 5 : Correction SSR — window is not defined
- Erreur : `ReferenceError: window is not defined` dans `getCurrentWindow()` (Titlebar.svelte) lors du rendu côté serveur.
- Cause : SvelteKit faisait du SSR ; l'API Tauri `window` n'existe que dans la webview.
- Fix : `export const ssr = false;` dans src/routes/+layout.ts (setup canonique Tauri + SvelteKit : SPA pure).
- Robustesse : Titlebar appelle désormais `getCurrentWindow()` paresseusement dans les handlers.
- Bonus : favicon.svg (marque M) dans static/ + lien dans app.html — corrige le 404 /favicon.ico.
- Vérification : `npm run build` OK, 13 pages HTML prerendues dans build/.

### Étape 6 : Workflows guidés (mock, avant SQLite)
- `docs/mantis-workflows.md` : chaîne Exposition → Incident → Action → RGPD ; hub Centre de posture.
- `src/lib/mock/posture.ts` : alertes, priorités, incidents, actions, démarches RGPD avec IDs croisés.
- `src/lib/workflow.css` : styles partagés des vues guidées.
- Centre de posture : score explicable, priorités et prochaines actions cliquables, alertes → `/incidents?id=`.
- Incidents / Actions / DPO : listes + panneau détail (quoi / pourquoi / ensuite), navigation croisée.
- Actions : état local session (à faire / en cours / faite). DPO : brouillon copiable, pas d’envoi auto.

### Étape 7 : Chaîne Dossier → Identité → Exposition
- Mock étendu : `folders`, `identities`, `exposures` + liens `folderId` / `exposureIds` sur incidents.
- Vues guidées : `/dossiers`, `/identites`, `/expositions` (liste + détail, filtres `?folder=`).
- Incidents affichent les expositions sources ; navigation croisée complète jusqu’aux actions / DPO.

### Étape 8 : UX guidante (mini-guides par vue)
- Composants : `GuideHeader`, `ReadOnlyField`, `NextStepBar`.
- Toutes les vues métier (posture, dossiers, identités, expositions, incidents, actions, DPO) :
  - question UX explicite + intro simple ;
  - résumé « en bref » / barre Ensuite avec CTA primaire ;
  - champs identité & contact DPO marqués lecture seule (Phase 2).
- Centre de posture : phrase d’interprétation du score + alertes avec « Comprendre et traiter → ».

### Étape 9 : Compléter le cockpit (vues secondaires guidées)
- Mock : `guides`, `watchRoutines`, `graphEdges`, `reportSnapshot`, `appPrinciples`.
- Vues : `/graphe` (relations cliquables), `/veille` (routines transparentes, 0 réseau), `/guides`, `/rapports` (aperçu), `/parametres` (principes + lecture seule).
- Plus de placeholder sidebar. Décision doc : finir le cockpit UX avant Tailwind / SQLite.

### Prochaines étapes
- [ ] Smoke test `npm run tauri dev` / `npm run build`.
- [ ] Phase 2 : SQLite + formulaires d’édition (identités, dossiers, contacts DPO).
- [ ] Tailwind : adosser le thème CSS existant (refactor, pas redesign).
- [ ] Puis Phase 3–4 (cartographie, OSINT contrôlé).

## Phase 2 — Modèle de données SQLite
- Schéma : folders, identities, exposures, incidents, actions, rgpd_requests, timeline_entries.
- Plugin SQL Tauri ou crate Rust.
- Commandes Tauri CRUD + déblocage des champs « lecture seule ».

## Phase 3+ — À planifier selon roadmap
