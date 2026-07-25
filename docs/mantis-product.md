# MANTIS POSTURE Product vision
# MANTIS POSTURE – Vision produit

MANTIS POSTURE est un cockpit décisionnel de posture numérique personnelle, installé en local sur la machine de l’utilisateur.
Le logiciel transforme des traces numériques dispersées (comptes, profils, fuites, mentions publiques) en un plan d’action compréhensible et mesurable.

## Objectifs principaux

- Savoir **ce qui est exposé** sur l’utilisateur (identités, comptes, coordonnées, profils publics, traces OSINT).
- Expliquer **pourquoi chaque exposition est importante** et quel risque elle porte (usurpation, phishing, harcèlement, compromission de compte, réputation).
- Proposer **quoi faire ensuite**, avec des actions concrètes et vérifiables.
- Réduire progressivement :
  - Les traces publiques inutiles.
  - Le volume de données personnelles accessibles en ligne.
  - Les risques d’usurpation d’identité, de phishing ciblé et de harcèlement.
- Donner une vue claire des **entreprises et services qui possèdent des données** sur l’utilisateur (cartographie des comptes et des fournisseurs).
- Automatiser autant que possible les **demandes RGPD** (accès, rectification, effacement, opposition, déréférencement), en gardant le contrôle local des modèles de requêtes.

## Ce que MANTIS remonte et surveille

MANTIS ne promet pas une surveillance totale d’Internet, mais il organise et automatise un ensemble de vérifications ciblées :

- **Résultats de moteurs de recherche** :
  - Recherches ciblées sur les noms, pseudos, e-mails, domaines et organisations de l’utilisateur.
  - Repérage de pages publiques parlant de l’utilisateur ou contenant ses coordonnées.
- **Fuites de données via des APIs autorisées** :
  - Vérification d’e-mails, domaines, pseudos, numéros de téléphone dans des bases de fuites exposées ou des services de breach intelligence.
  - Regroupement des occurrences (dans quels leaks, à quelles dates, avec quelle gravité). [web:250][web:253][web:261][web:260]
- **Automations OSINT de base** :
  - Recherche de mentions indexées via des APIs de recherche.
  - Surveillance de pages publiques ajoutées par l’utilisateur (profils, pages de résultats, documents).
  - Recherche de traces publiques liées à des pseudos ou identifiants.
  - Vérification de domaines : DNS, certificats TLS, sous-domaines publics.

L’outil reste strictement centré sur **l’utilisateur ou les dossiers autorisés** : il ne doit jamais être utilisé pour cibler des tiers sans consentement explicite.

## Concepts métier

- **Dossiers** :
  - Conteneurs thématiques : personnel, professionnel, activité, domaine, projet, etc.
  - Permettent d’organiser les identités et les expositions par contexte (ex. “emploi actuel”, “ancienne entreprise”, “side project”).
- **Identités** :
  - Noms et variantes.
  - Adresses e-mail.
  - Numéros de téléphone.
  - Pseudos et handles.
  - Domaines et sous-domaines.
  - Organisations et rôles.
  - URLs et profils publics (réseaux sociaux, blogs, profils d’entreprise).
- **Résultats OSINT** :
  - Signaux bruts issus de sources autorisées (moteurs de recherche, APIs de fuite, plateformes OSINT, vérifications de domaines). [web:250][web:251][web:261]
- **Expositions** :
  - Résultats jugés pertinents pour la posture numérique (par exemple : adresse e-mail présente dans un leak, numéro de téléphone visible dans un PDF, profil public exposant trop de données).
- **Incidents** :
  - Expositions qui nécessitent suivi et traitement (ex. fuite de credentials, usurpation avérée, compte compromis, menace de harcèlement).
- **Actions** :
  - Tâches à effectuer pour réduire le risque :
    - Sécuriser un compte (changement de mot de passe, activation MFA).
    - Modifier des paramètres de confidentialité.
    - Supprimer ou réduire une donnée publique.
    - Envoyer une demande RGPD.
  - Chaque action possède :
    - Priorité.
    - Difficulté.
    - Échéance.
    - Preuve de réalisation.
    - Vérification finale.
- **Démarches RGPD** :
  - Préparation, suivi et archivage des demandes :
    - Accès (Data Subject Access Request).
    - Rectification.
    - Effacement.
    - Opposition.
    - Déréférencement.
  - MANTIS génère les éléments nécessaires (modèles de courrier, synthèse des données en jeu, contacts des DPO) et aide à automatiser la préparation, tout en laissant l’utilisateur envoyer les demandes via des canaux qu’il contrôle. [web:254][web:256][web:259][web:262]
- **Chronologie** :
  - Historique local des recherches, détections, décisions, actions et revues :
    - Quand telle fuite a été détectée.
    - Quelles actions ont été entreprises.
    - Quand un rapport de posture a été généré.
    - Quand une demande RGPD a été envoyée et quand une réponse a été reçue.

## Ensemble des entreprises et services qui possèdent des données

MANTIS doit aider l’utilisateur à savoir **où sont ses données et où il a des comptes** :

- Regrouper les services et entreprises :
  - Fournisseurs de mail, réseaux sociaux, plateformes professionnelles, hébergeurs, banques, services de mobilité, etc.
- Associer chaque identifiant (e-mail, pseudo, téléphone, domaine) aux plateformes où il est utilisé.
- Identifier les **entités qui stockent ou exploitent les données** de l’utilisateur :
  - Sites où il a un compte actif ou ancien.
  - Services tiers (tracking, newsletters, partenaires).
- Préparer les informations nécessaires pour une démarche RGPD :
  - Coordonnées du DPO ou du service privacy.
  - Historique synthétique des données (type, contexte).
  - Modèle de demande adapté (accès, effacement, opposition, déréférencement).

L’objectif n’est pas d’automatiser entièrement la relation juridique, mais de donner à l’utilisateur **une vue claire de son paysage de données** et de lui **économiser du temps** dans la préparation et le suivi de ses requêtes.

## Interface utilisateur
## Interface utilisateur

MANTIS est conçu comme un **cockpit analytique premium**, inspiré des interfaces de type Palantir : sobres, modernes, lisibles, centrées sur l’analyse.

- **Sidebar** :
  - Centre de posture.
  - Dossiers.
  - Graphe.
  - Identités.
  - Veille.
  - Expositions.
  - Incidents.
  - Actions.
  - DPO.
  - Guides.
  - Rapports.
  - Paramètres.

- **Centre de posture** :
  - Score explicable de posture numérique (avec détails sur les facteurs positifs et négatifs).
  - Liste de priorités (actions les plus importantes).
  - Alertes classées par niveau (information, faible, modérée, élevée, critique).
  - Prochaines actions recommandées.

- **Graphe** :
  - Visualisation des liens entre identités, résultats OSINT, expositions, incidents et actions.

- **Tableaux et détails** :
  - Tableaux denses par dossier et par type d’objet (identités, expositions, incidents, actions).
  - Filtres, tri et recherche.
  - Panneau de détails pour chaque élément (fiche d’identité, fiche d’exposition, fiche d’incident, fiche d’action).

- **Rapports** :
  - Génération de rapports HTML/PDF de posture numérique pour un dossier ou pour l’ensemble du profil.
  - Vue synthétique des risques, des fuites, des actions et des démarches RGPD en cours ou réalisées.

## Esthétique et expérience utilisateur

L’esthétique de MANTIS doit évoquer un **outil moderne de pilotage de données**, proche de l’univers Palantir, mais centré sur une personne :

- **Thème** :
  - Sombre graphite/ardoise, avec contrastes maîtrisés.
  - Palette limitée, élégante, avec quelques accents de couleur pour les alertes et les états (par exemple, verts pour “OK”, jaunes/oranges pour attention, rouges pour critique).

- **Typographie et lisibilité** :
  - Typographie sans-serif moderne.
  - Hiérarchie visuelle claire (titres, sous-titres, métadonnées).
  - Écrans pensés pour afficher beaucoup d’information sans surcharge visuelle.

- **Composants** :
  - Cards et tableaux denses pour structurer les données.
  - Panneaux latéraux de détails, pour approfondir une identité, une exposition, un incident ou une action sans quitter la vue principale.
  - Graphes et visualisations sobres (nœuds, liens, niveaux d’alerte), sans animations inutiles.

- **Ton général** :
  - Professionnel, sérieux, orienté analyse, sans esthétique “hacker/cyberpunk”.
  - L’interface doit inspirer confiance et donner un sentiment de contrôle et de clarté sur sa posture numérique.

Cette esthétique vise à positionner MANTIS comme un **outil de décision moderne**, pas comme un gadget de sécurité : l’utilisateur doit se sentir dans un cockpit de données personnel, à la manière des plateformes d’analyse haut de gamme.