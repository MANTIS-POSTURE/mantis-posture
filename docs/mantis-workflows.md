# MANTIS POSTURE — Workflows & plan UX

Ce document décrit **comment le cockpit guide l’utilisateur**.
La logique et le raisonnement restent ici — pas dans le code UI.

Principe : chaque écran est un **mini-guide**, pas une simple liste de données.
Trois questions partout : **Quoi ?** → **Pourquoi ?** → **Ensuite ?**

---

## Chaîne métier (rappel)

```
Dossier → Identité → Exposition → Incident → Action → (RGPD si besoin) → Score
```

---

## OSINT Défensif & Posture Numérique

MANTIS POSTURE intègre l'OSINT de manière défensive et centrée sur l'utilisateur, contrairement aux outils d'investigation classique.

### Positionnement vs outils existants (HIBP, holehe, OSINTLeak, Seekr)

- **Vs Seekr / Toolkits** : MANTIS n'est pas une boîte à outils OSINT générique. C'est un cockpit de pilotage. Les outils externes sont abstraits derrière des "Modules de veille" que l'utilisateur lance d'un clic.
- **Vs HIBP / OSINTLeak** : MANTIS ne se contente pas d'afficher une liste de fuites. Il transforme la fuite en un **Incident** suivi, avec une **Action** de remédiation guidée (ex: "Changer le mot de passe de ce service").
- **Confidentialité** : MANTIS est local-first. Les requêtes vers des APIs externes (si activées) doivent minimiser les données transmises (ex: hash SHA-1 de l'e-mail pour HIBP). Aucune donnée de l'utilisateur n'est envoyée à un serveur MANTIS.

### Workflow OSINT dans MANTIS

1. **Cible** : L'utilisateur configure ses Identités (e-mails, pseudos, domaines) dans les Dossiers.
2. **Module** : L'utilisateur (ou une routine planifiée) lance un Module OSINT (ex: "Vérification fuites e-mails").
3. **Signal** : Le module interroge une source (API, moteur de recherche) et remonte un résultat brut.
4. **Exposition** : MANTIS filtre le signal. S'il est pertinent, il crée une entrée dans `exposures` (statut: `nouvelle`).
5. **Incident** : Si l'exposition est grave (ex: fuite de mot de passe), MANTIS propose de créer un `incident`.
6. **Action** : L'incident déclenche une action (ex: "Rotation des mots de passe", "Activation MFA").

---

## Objectifs UX par vue

### Centre de posture (`/posture`)

- **Question principale :** « Où en suis-je, et que dois-je faire en premier ? »
- **Objectif UX :** hub de décision — score compréhensible, alertes classées, 2–3 priorités, prochaines actions.
- **Parcours :**
  1. Lire le score + facteurs (positif / négatif) en langage simple.
  2. Cliquer une alerte → Incident (avec contexte).
  3. Cliquer une priorité / prochaine action → Action guidée.
  4. Si besoin juridique → raccourci vers DPO.
- **Liens sortants :** → Incidents, → Actions, (plus tard) → DPO, → Guides.

### Incidents (`/incidents`)

- **Question principale :** « Qu’est-ce qui me demande un vrai suivi, et pourquoi ? »
- **Objectif UX :** expliquer chaque incident (quoi / pourquoi / impact / confiance) et proposer **une** prochaine étape claire.
- **Parcours :**
  1. Choisir un incident dans la liste (gravité visible).
  2. Lire le panneau détail (guide, pas fiche technique brute).
  3. Ouvrir l’action recommandée, ou la démarche DPO, ou remonter aux expositions sources.
- **Liens :** ← Centre / Expositions · → Actions · → DPO · → Expositions.

### Actions (`/actions`)

- **Question principale :** « Que dois-je faire concrètement, et comment ? »
- **Objectif UX :** checklist de remédiation (priorité, difficulté, échéance, étapes, preuve attendue).
- **Parcours :**
  1. Ouvrir une action (depuis le centre ou un incident).
  2. Suivre les étapes numérotées hors MANTIS si besoin (ex. LinkedIn).
  3. Marquer en cours / faite (sans jamais saisir de secrets).
  4. Si RGPD : basculer vers DPO.
- **Liens :** ← Incidents · → DPO · → Centre.

### DPO / RGPD (`/dpo`)

- **Question principale :** « Comment préparer ma demande de droits, sans me tromper ? »
- **Objectif UX :** assistant en étapes (type → cible → données → brouillon → envoi manuel).
- **Parcours :**
  1. Voir le statut de la démarche.
  2. Vérifier cible / contact (lecture seule pour l’instant).
  3. Copier le brouillon ; envoyer soi-même.
  4. Marquer « j’ai envoyé » pour le suivi.
- **Règle :** jamais d’envoi automatique.
- **Liens :** ← Actions / Incidents · → Centre.

### Dossiers (`/dossiers`)

- **Question principale :** « Dans quel contexte je regarde ma posture ? »
- **Objectif UX :** choisir un périmètre (personnel, emploi…) puis voir identités / expositions / incidents liés.
- **Parcours :** dossier → identités ou expositions → suite de la chaîne.
- **Liens :** → Identités · → Expositions · → Incidents.

### Identités (`/identites`)

- **Question principale :** « Quelles traces me concernent (noms, e-mails, profils) ? »
- **Objectif UX :** inventaire clair, sans secrets ; lien vers les expositions qui touchent cette identité.
- **Parcours :** identité → expositions liées → incident si suivi.
- **Éditabilité :** valeurs en **lecture seule** pour l’instant (voir section Édition).
- **Liens :** ← Dossiers · → Expositions.

### Expositions (`/expositions`)

- **Question principale :** « Qu’est-ce qui a été trouvé sur moi, et est-ce important ? »
- **Objectif UX :** transformer une trace en compréhension + décision (suivre / accepter / réduire via incident).
- **Parcours :** exposition → incident → action / DPO.
- **Liens :** ← Identités / Dossiers · → Incidents · → Identités.

### Graphe (`/graphe`)

- **Question principale :** « Comment mes traces sont-elles reliées entre elles ? »
- **Objectif UX :** carte relationnelle simple (pas un outil d’analyste) — voir les liens Identité → Exposition → Incident → Action / DPO.
- **Parcours :** cliquer un nœud / lien → fiche correspondante.
- **Phase 1 :** liste de relations cliquables (pas de moteur de graphe lourd).

### Veille (`/veille`)

- **Question principale :** « Qu’est-ce qui tourne en arrière-plan pour me surveiller ? »
- **Objectif UX :** transparence sur les routines (fréquence, sources, données transmises) — rassurer avant l’OSINT réel.
- **Parcours :** lire une routine → comprendre qu’elle est planifiée / désactivée → aller au Centre ou aux Identités.
- **Phase 1 :** mock uniquement, **aucune requête réseau**.

### Guides (`/guides`)

- **Question principale :** « Comment m’y prendre si je ne suis pas expert ? »
- **Objectif UX :** mini-tutoriels liés aux actions (MFA, confidentialité, RGPD).
- **Parcours :** guide → action ou DPO associée.

### Rapports (`/rapports`)

- **Question principale :** « Puis-je résumer ma posture pour moi (ou plus tard pour un tiers de confiance) ? »
- **Objectif UX :** aperçu de synthèse ; export réel plus tard.
- **Phase 1 :** aperçu mock + CTA vers les priorités du Centre.

### Paramètres (`/parametres`)

- **Question principale :** « Comment MANTIS traite mes données, et que puis-je configurer ? »
- **Objectif UX :** afficher les principes (local-first, pas de télémétrie, pas de secrets) ; réglages avancés plus tard.
- **Éditabilité :** principes en lecture ; toggles réels avec SQLite / OSINT.

---

## Parcours utilisateur principaux

### P1 — Alerte → remédiation (chemin rapide)

```
Centre de posture (alerte)
  → Incident (comprendre)
    → Action (étapes guidées)
      → [option] DPO (brouillon)
        → retour Centre (score / priorités)
```

### P2 — Exploration par contexte

```
Dossiers → Identités → Expositions → Incident → Action / DPO
```

### P3 — Droit RGPD direct

```
Centre (priorité RGPD) ou Action liée
  → DPO (étapes + brouillon)
    → envoi manuel hors MANTIS
      → statut « envoyée »
```

---

## Édition des champs (état actuel → futur)

### Aujourd’hui (Phase 1 — mock)

| Zone | Exemples | UI attendue |
|------|----------|-------------|
| Identités | nom, e-mail, téléphone, pseudo, URL | Affichage lecture seule + mention « Non éditable pour l’instant » |
| DPO | contact privacy / e-mail DPO, cible | Lecture seule + « Saisie prévue en Phase 2 » |
| Score / alertes / expositions OSINT | facteurs, résumés de fuites | Lecture seule (données système / mock) |
| Actions / DPO suivi | statut action, statut envoi | **Interactif** (état local session) |
| Brouillon RGPD | texte généré | Copie autorisée ; édition libre du texte = Phase 2+ |

### Plus tard (Phase 2 — SQLite)

- **Identités :** formulaire « Ajouter / modifier » (label, type, valeur) avec validation :
  - e-mail : format basique ;
  - téléphone : affichage libre, pas de secret ;
  - refus explicite des champs mot de passe / MFA / IBAN / pièce d’identité.
- **Dossiers :** créer / renommer / archiver.
- **DPO :** éditer cible, e-mail contact, type de demande ; régénérer le brouillon ; persister dates d’envoi / réponse.
- **Lien technique :** formulaires Svelte → commandes Tauri → tables `identities`, `folders`, `rgpd_requests` (pas de secrets en base).

---

## Plan UI — état & suite

### Fait (étapes 6–8)

- Composants `GuideHeader`, `ReadOnlyField`, `NextStepBar`.
- Vues métier guidées : posture, dossiers, identités, expositions, incidents, actions, DPO.

### En cours / immédiat (étape 9)

Compléter le **cockpit Phase 1** : Graphe, Veille, Guides, Rapports, Paramètres en mode mini-guide mock.
Objectif : plus aucun écran « Vue à venir » dans la sidebar.

### Ensuite (ordre recommandé)

1. **Stabiliser Phase 1** — `npm run tauri dev` / build ; petit polish UX si besoin.
2. **Phase 2 SQLite** — schéma + CRUD dossiers/identités ; débloquer l’édition des champs marqués lecture seule.
3. **Tailwind** — adosser le thème existant (refactor CSS), sans changer le langage UX.
4. **Phases 3–4** — cartographie entreprises, OSINT contrôlé (après persistance).

### Décision produit

Priorité = **cockpit compréhensible et guidant** avant persistance et OSINT.
Tailwind et SQLite restent importants mais ne doivent pas interrompre la complétion des 12 vues sidebar.

---

## Données mock (rappel)

Source unique : `src/lib/mock/posture.ts` — IDs croisés + guides, routines de veille, arêtes de graphe, aperçu rapport.

---

## Hors scope de ce document

- Schéma SQLite et commandes Tauri (Phase 2).
- Collecte OSINT réelle (Phase 4).
- Calcul Rust du score (Phase 5).
- Génération PDF / envoi mail (Phases 6–7).
```
