# MANTIS POSTURE Roadmap
# MANTIS POSTURE – Roadmap de développement pour les agents IA

Cette Roadmap définit les grandes phases de construction de MANTIS POSTURE.
Elle sert de guide pour les agents IA et les humains : on avance par blocs cohérents plutôt que par bricolage ponctuel.

---

## Phase 1 – Socle local-first et cockpit UI

**Objectif :** disposer d’une application desktop locale stable avec un cockpit de base.

- Mettre en place l’application Tauri 2 + SvelteKit + TypeScript + Tailwind.
- Stabiliser la commande `npm run tauri dev` sur Windows.
- Implémenter :
  - La sidebar complète (Centre de posture, Dossiers, Graphe, Identités, Veille, Expositions, Incidents, Actions, DPO, Guides, Rapports, Paramètres).
  - Une page “Centre de posture” avec données fictives :
    - Score de posture fictif.
    - Priorités mock.
    - Alertes mock classées par niveau.
    - Prochaines actions mock.
- Assurer le thème sombre moderne, type cockpit Palantir, sans télémétrie ni analytics.

**Résultat attendu :**
- Un client desktop local qui s’ouvre.
- Une interface claire permettant de naviguer entre les sections, même sans logique métier complète.

---

## Phase 2 – Modèle de données et stockage local

**Objectif :** disposer d’un modèle de données robuste pour représenter la posture numérique.

- Concevoir le schéma SQLite local :
  - `folders` (dossiers).
  - `identities` (identités).
  - `exposures` (expositions).
  - `incidents`.
  - `actions`.
  - `rgpd_requests`.
  - `timeline_entries`.
- Intégrer SQLite dans Tauri (plugin SQL ou crate Rust) en respectant le principe local-first et en excluant tout stockage de secrets.
- Implémenter des commandes Tauri pour :
  - Créer, lire, mettre à jour et archiver les dossiers et identités.
  - Enregistrer les expositions, incidents, actions, démarches RGPD et événements de chronologie.

**Résultat attendu :**
- Une base locale cohérente.
- Un cockpit qui manipule des données persistantes (fictives ou de test), avec un historique structuré.

---

## Phase 3 – Cartographie des comptes et des entreprises qui détiennent les données

**Objectif :** donner à l’utilisateur une vue claire de **qui possède ses données** et où il a des comptes.

- Construire une vue “entreprises / services” :
  - Regrouper les plateformes par identifiants (e-mails, pseudos, numéros de téléphone, domaines).
  - Lister les comptes connus ou déduits (sites, applications, services) par dossier et par identité.
- Permettre à l’utilisateur :
  - D’ajouter / confirmer les services où il a un compte.
  - De marquer les comptes comme actifs, anciens, ou supprimés.
- Préparer les liens avec les démarches RGPD :
  - Associer à chaque service les coordonnées du DPO ou du service privacy (quand disponibles).
  - Garder une fiche synthétique des données en jeu (types de données, contexte, dossier de rattachement).

**Résultat attendu :**
- Une cartographie lisible des entreprises et services qui détiennent des données sur l’utilisateur.
- Une base pour déclencher des démarches RGPD ciblées.

---

## Phase 4 – Collecte OSINT et détection de fuites

**Objectif :** remonter de façon structurée les traces et fuites publiques liées aux identités de l’utilisateur.

- Définir un cadre OSINT minimal, aligné avec les bonnes pratiques :
  - Limiter la collecte aux identifiants de l’utilisateur ou à des dossiers explicitement autorisés.
  - Respecter les contraintes légales et éthiques des sources utilisées. [web:269][web:271][web:275][web:272]
- Implémenter des modules de veille :
  - Vérification de fuites d’e-mails, domaines, pseudos, numéros de téléphone via des APIs autorisées (breach intelligence, data leak check).
  - Recherche de traces publiques via des moteurs de recherche (mentions, profils, documents).
  - Vérification de domaines : DNS, certificats TLS, sous-domaines publics.
  - Surveillance de pages publiques ajoutées par l’utilisateur.
- Structurer le cycle OSINT :
  - Planification (définir les identifiants et dossiers à surveiller).
  - Collecte (OSINT ciblé).
  - Traitement (dé-duplication, filtrage, score de pertinence).
  - Analyse (qualification en expositions ou incidents).
  - Diffusion vers le cockpit (alertes, actions recommandées).

**Résultat attendu :**
- Un flux régulier de signaux OSINT sur les identités de l’utilisateur.
- Un centre d’alertes capable de distinguer l’information des vraies expositions.
- Très peu de bruit : les doublons et données déjà traitées doivent être filtrés.

---

## Phase 5 – Score de posture et centre d’alertes avancé

**Objectif :** transformer les données en **indicateurs de posture** et en alertes actionnables.

- Définir un modèle de score de posture :
  - Variables : nombre et gravité des expositions, niveau de protection des comptes, volume de traces publiques, état des actions de remédiation.
  - Ponderations : focaliser sur les risques les plus critiques (usurpation, compromission de comptes, harcèlement).
- Implémenter :
  - Une fonction Rust qui calcule le score de posture à partir des données SQLite.
  - Une commande Tauri qui expose ce score au frontend.
- Structurer le centre d’alertes :
  - Classes de gravité : information, faible, modérée, élevée, critique.
  - Pour chaque alerte :
    - Ce qui a été trouvé.
    - Pourquoi c’est important.
    - Niveau de confiance.
    - Impact possible.
    - Action recommandée.
- Assurer que les **alertes critiques** :
  - Sont rares.
  - Sont directement actionnables (elles débouchent sur une action concrète dans le cockpit).

**Résultat attendu :**
- Un score de posture explicable.
- Un centre d’alertes utile qui met en avant ce qui mérite vraiment une action.

---

## Phase 6 – Automatisation des démarches RGPD

**Objectif :** aider l’utilisateur à exercer ses droits (accès, rectification, effacement, opposition, déréférencement) de manière structurée, avec un minimum de friction.

- Construire un module “DPO / RGPD” :
  - Fiches par entreprise / service avec les coordonnées du DPO ou du service privacy.
  - Historique des demandes (type, date, statut).
- Automatiser la préparation des demandes :
  - Générer des modèles de requêtes RGPD (DSAR, effacement, opposition) pré-remplis à partir des données du dossier.
  - Préparer les listes de données concernées (types, contexte, identités).
  - Proposer des canaux d’envoi (mail, formulaire, courrier) sans imposer de cloud.
- Assurer le suivi :
  - Enregistrer la date d’envoi.
  - Enregistrer les réponses.
  - Mettre à jour la posture (ex. données supprimées, rectifiées, oppositions prises en compte).

**Résultat attendu :**
- Un cockpit où l’utilisateur voit :
  - Les entreprises qui détiennent ses données.
  - Les démarches RGPD déjà réalisées.
  - Les démarches recommandées ou en attente.
- Une expérience où la préparation d’une demande RGPD est largement automatisée, mais l’envoi reste sous contrôle de l’utilisateur.

---

## Phase 7 – Guides, routines et rapports

**Objectif :** rendre MANTIS pédagogique et exploitable dans le temps.

- Guides de remédiation :
  - MFA, sécurisation des méthodes de récupération.
  - Alias e-mail et compartimentation des identités.
  - Confidentialité des réseaux sociaux.
  - Nettoyage de vieux comptes.
  - Démarches RGPD, phishing, mises à jour, sauvegardes, protection de domaine.
- Routines :
  - Vérifications quotidiennes, hebdomadaires, mensuelles, trimestrielles ou personnalisées.
  - Chaque routine affiche :
    - Sources utilisées.
    - Données transmises.
    - Fréquence.
    - Coût/quota éventuel.
    - Dernière exécution.
    - Prochaine exécution.
    - Erreurs éventuelles.
- Rapports :
  - Génération de rapports HTML/PDF de posture :
    - Par dossier.
    - Pour l’ensemble du profil.
  - Synthèse des risques, fuites, actions, démarches RGPD.

**Résultat attendu :**
- Un outil qui ne se limite pas à détecter des fuites, mais qui accompagne l’utilisateur dans la réduction de son exposition, la compréhension de ses risques, et l’exercice de ses droits.

---

Les agents IA et les développeurs doivent toujours se situer par rapport à cette Roadmap :
- “Phase 1–2” pour le socle applicatif et la base de données.
- “Phase 3–4” pour la cartographie et la collecte OSINT.
- “Phase 5–6–7” pour les scores, alertes, RGPD, guides, routines et rapports.