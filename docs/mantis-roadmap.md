# MANTIS POSTURE Roadmap
# MANTIS POSTURE – Roadmap de développement pour les agents IA

Cette Roadmap définit les grandes phases de construction de MANTIS POSTURE.
Elle sert de guide pour les agents IA et les humains : on avance par blocs cohérents plutôt que par bricolage ponctuel.

---

## Phase 1 – Socle local-first et cockpit UI

**Objectif :** disposer d’une application desktop locale stable avec un cockpit de base.

- [x] Mettre en place l’application Tauri 2 + SvelteKit + TypeScript.
- [x] Stabiliser la commande `npm run tauri dev` sur Windows.
- [x] Implémenter la sidebar complète (12 sections).
- [x] Assurer le thème sombre moderne, type cockpit Palantir (variables CSS, glassmorphism).
- [x] Créer des vues mock guidées pour toutes les sections.

**Résultat attendu :**
- [x] Un client desktop local qui s’ouvre.
- [x] Une interface claire permettant de naviguer entre les sections, même sans logique métier complète.

---

## Phase 2 – Modèle de données et stockage local

**Objectif :** disposer d’un modèle de données robuste pour représenter la posture numérique.

- [x] Concevoir le schéma SQLite local (`0001_init.sql`).
- [x] Intégrer SQLite dans Tauri (crate Rust `rusqlite`).
- [x] Implémenter l'initialisation et le seed de données au démarrage.
- [x] Créer les commandes Tauri pour lister les entités (folders, identities, exposures, incidents, actions, rgpd, timeline, score).
- [x] Connecter le Centre de posture (`/posture`) et la vue Incidents (`/incidents`) à SQLite.
- [ ] Connecter les vues restantes à SQLite :
  - [x] Expositions (`/expositions`)
  - [ ] Actions (`/actions`)
  - [ ] Dossiers (`/dossiers`)
  - [ ] Identités (`/identites`)
  - [ ] DPO (`/dpo`)
- [ ] Débloquer l'édition des champs (CRUD complet).

**Résultat attendu :**
- Une base locale cohérente.
- Un cockpit qui manipule des données persistantes (fictives ou de test), avec un historique structuré.
- Toutes les vues utilisent l'API SQLite réelle et le thème glassmorphism.

---

## Phase 3 – Cartographie des comptes et des entreprises qui détiennent les données

**Objectif :** donner à l’utilisateur une vue claire de **qui possède ses données** et où il a des comptes.

- Construire une vue “entreprises / services”.
- Permettre à l’utilisateur d’ajouter / confirmer les services.
- Préparer les liens avec les démarches RGPD.

**Résultat attendu :**
- Une cartographie lisible des entreprises et services.
- Une base pour déclencher des démarches RGPD ciblées.

---

## Phase 4 – Collecte OSINT et détection de fuites

**Objectif :** remonter de façon structurée les traces et fuites publiques.

- Définir un cadre OSINT minimal.
- Implémenter des modules de veille (fuites, mentions, domaines).
- Structurer le cycle OSINT (Planification → Diffusion).

**Résultat attendu :**
- Un flux régulier de signaux OSINT.
- Un centre d’alertes capable de distinguer l’information des vraies expositions.

---

## Phase 5 – Score de posture et centre d’alertes avancé

**Objectif :** transformer les données en **indicateurs de posture** et en alertes actionnables.

- Définir un modèle de score de posture.
- Implémenter une fonction Rust qui calcule le score.
- Structurer le centre d’alertes (classes de gravité, actions recommandées).

**Résultat attendu :**
- Un score de posture explicable.
- Un centre d’alertes utile qui met en avant ce qui mérite vraiment une action.

---

## Phase 6 – Automatisation des démarches RGPD

**Objectif :** aider l’utilisateur à exercer ses droits de manière structurée.

- Construire un module “DPO / RGPD”.
- Automatiser la préparation des demandes (modèles pré-remplis).
- Assurer le suivi (envoi, réponses).

**Résultat attendu :**
- Un cockpit où l’utilisateur voit les démarches en cours.
- Une expérience où la préparation d’une demande RGPD est largement automatisée.

---

## Phase 7 – Guides, routines et rapports

**Objectif :** rendre MANTIS pédagogique et exploitable dans le temps.

- Guides de remédiation (MFA, confidentialité, etc.).
- Routines de vérification.
- Rapports HTML/PDF de posture.

**Résultat attendu :**
- Un outil qui accompagne l’utilisateur dans la réduction de son exposition.
