/** Mock data for Phase 1 guided workflows. No secrets, no persistence. */

export type AlertLevel = 'info' | 'low' | 'moderate' | 'high' | 'critical';
export type Severity = 'faible' | 'modérée' | 'élevée' | 'critique';
export type Priority = 'basse' | 'moyenne' | 'haute' | 'critique';
export type Difficulty = 'facile' | 'moyenne' | 'difficile';
export type ActionStatus = 'a_faire' | 'en_cours' | 'faite';
export type RgpdType =
	| 'acces'
	| 'rectification'
	| 'effacement'
	| 'opposition'
	| 'dereferencement';
export type RgpdStatus = 'brouillon' | 'prete' | 'envoyee' | 'repondue';
export type IdentityKind =
	| 'nom'
	| 'email'
	| 'telephone'
	| 'pseudo'
	| 'domaine'
	| 'url';
export type ExposureKind = 'profil_public' | 'fuite' | 'annuaire' | 'mention';
export type ExposureStatus = 'nouvelle' | 'en_suivi' | 'acceptee' | 'reduite';

export interface Folder {
	id: string;
	name: string;
	context: string;
	identityIds: string[];
}

export interface Identity {
	id: string;
	label: string;
	kind: IdentityKind;
	value: string;
	folderId: string;
	notes?: string;
}

export interface Exposure {
	id: string;
	title: string;
	kind: ExposureKind;
	severity: Severity;
	status: ExposureStatus;
	discoveredAt: string;
	source: string;
	what: string;
	why: string;
	identityIds: string[];
	folderId: string;
	incidentId?: string;
}

export interface Alert {
	id: string;
	level: AlertLevel;
	title: string;
	summary: string;
	/** Linked incident when the alert is actionable */
	incidentId?: string;
}

export interface Incident {
	id: string;
	title: string;
	severity: Severity;
	discoveredAt: string;
	what: string;
	why: string;
	impact: string;
	confidence: string;
	nextStep: string;
	actionIds: string[];
	exposureIds: string[];
	rgpdId?: string;
	folderId: string;
	dossier: string;
}

export interface ActionItem {
	id: string;
	title: string;
	priority: Priority;
	difficulty: Difficulty;
	deadline: string;
	status: ActionStatus;
	guidance: string[];
	proofExpected: string;
	incidentId?: string;
	rgpdId?: string;
	folderId: string;
	dossier: string;
}

export interface RgpdRequest {
	id: string;
	type: RgpdType;
	target: string;
	dpoContact: string;
	status: RgpdStatus;
	identities: string[];
	dataSummary: string;
	draftPreview: string;
	createdAt: string;
	incidentId?: string;
	actionId?: string;
}

export const postureScore = {
	value: 72,
	label: 'Score de posture',
	factorsPositive: [
		'MFA activée sur le compte e-mail principal',
		'Aucun incident critique ouvert'
	],
	factorsNegative: [
		'E-mail professionnel visible sur un profil public',
		'2 fuites anciennes non remédiées'
	]
};

export const alerts: Alert[] = [
	{
		id: 'alert-info-1',
		level: 'info',
		title: 'Aucune exposition critique',
		summary: 'Aucune exposition critique détectée sur le dossier personnel.'
	},
	{
		id: 'alert-low-1',
		level: 'low',
		title: 'Fuites anciennes',
		summary: '2 fuites anciennes identifiées dans des bases publiques.',
		incidentId: 'inc-leaks'
	},
	{
		id: 'alert-mod-1',
		level: 'moderate',
		title: 'E-mail pro exposé',
		summary: 'Profil LinkedIn exposant l’adresse e-mail professionnelle.',
		incidentId: 'inc-linkedin'
	}
];

export const folders: Folder[] = [
	{
		id: 'folder-perso',
		name: 'Personnel',
		context: 'Identités et traces hors travail',
		identityIds: ['id-name', 'id-email-perso', 'id-phone']
	},
	{
		id: 'folder-job',
		name: 'Emploi actuel',
		context: 'Présence professionnelle et comptes liés au travail',
		identityIds: ['id-email-pro', 'id-linkedin']
	}
];

export const identities: Identity[] = [
	{
		id: 'id-name',
		label: 'Nom complet',
		kind: 'nom',
		value: 'Alex Martin',
		folderId: 'folder-perso'
	},
	{
		id: 'id-email-perso',
		label: 'E-mail personnel',
		kind: 'email',
		value: 'alex.martin.perso@example.com',
		folderId: 'folder-perso',
		notes: 'Utilisé pour comptes grand public'
	},
	{
		id: 'id-phone',
		label: 'Téléphone mobile',
		kind: 'telephone',
		value: '+33 6 ·· ·· ·· 42',
		folderId: 'folder-perso',
		notes: 'Affichage partiel — pas de secret'
	},
	{
		id: 'id-email-pro',
		label: 'E-mail professionnel',
		kind: 'email',
		value: 'a.martin@entreprise-exemple.example',
		folderId: 'folder-job'
	},
	{
		id: 'id-linkedin',
		label: 'Profil LinkedIn',
		kind: 'url',
		value: 'https://www.linkedin.com/in/exemple-alex-martin',
		folderId: 'folder-job'
	}
];

export const exposures: Exposure[] = [
	{
		id: 'exp-linkedin-email',
		title: 'E-mail pro sur profil LinkedIn',
		kind: 'profil_public',
		severity: 'modérée',
		status: 'en_suivi',
		discoveredAt: '2026-07-20',
		source: 'Profil public (observation manuelle mock)',
		what: 'L’adresse e-mail professionnelle apparaît dans la section contact du profil.',
		why: 'Facilite phishing ciblé et corrélation d’identités.',
		identityIds: ['id-email-pro', 'id-linkedin'],
		folderId: 'folder-job',
		incidentId: 'inc-linkedin'
	},
	{
		id: 'exp-leak-2019',
		title: 'Occurrence e-mail — fuite 2019',
		kind: 'fuite',
		severity: 'élevée',
		status: 'en_suivi',
		discoveredAt: '2026-07-18',
		source: 'Breach intelligence (résumé mock)',
		what: 'E-mail personnel signalé dans une fuite datée 2019. Aucun mot de passe ni dump stocké.',
		why: 'Risque si réutilisation d’identifiants sur des comptes encore actifs.',
		identityIds: ['id-email-perso'],
		folderId: 'folder-perso',
		incidentId: 'inc-leaks'
	},
	{
		id: 'exp-leak-2021',
		title: 'Occurrence e-mail — fuite 2021',
		kind: 'fuite',
		severity: 'élevée',
		status: 'nouvelle',
		discoveredAt: '2026-07-18',
		source: 'Breach intelligence (résumé mock)',
		what: 'Même e-mail personnel dans une seconde base (2021). Résumé uniquement.',
		why: 'Renforce la priorité de revue des comptes liés.',
		identityIds: ['id-email-perso'],
		folderId: 'folder-perso',
		incidentId: 'inc-leaks'
	},
	{
		id: 'exp-directory',
		title: 'Ancienne adresse dans un annuaire',
		kind: 'annuaire',
		severity: 'faible',
		status: 'en_suivi',
		discoveredAt: '2026-07-15',
		source: 'Annuaire web public (mock)',
		what: 'Fiche nominative avec ancienne adresse postale.',
		why: 'Donnée personnelle inutilement exposée.',
		identityIds: ['id-name'],
		folderId: 'folder-perso',
		incidentId: 'inc-directory'
	}
];

export const priorities = [
	{
		id: 'prio-1',
		rank: 1,
		title: 'Masquer l’e-mail professionnel sur LinkedIn',
		reason: 'Réduit le risque de phishing ciblé et d’usurpation.',
		actionId: 'act-linkedin-email'
	},
	{
		id: 'prio-2',
		rank: 2,
		title: 'Traiter les fuites d’identifiants anciennes',
		reason: 'Vérifier que les mots de passe concernés ont bien été changés (sans les stocker ici).',
		actionId: 'act-review-leaks'
	},
	{
		id: 'prio-3',
		rank: 3,
		title: 'Préparer une demande d’effacement',
		reason: 'Un annuaire expose encore une ancienne adresse.',
		actionId: 'act-rgpd-directory'
	}
];

export const nextActions = [
	{
		id: 'next-1',
		label: 'Modifier la confidentialité LinkedIn',
		actionId: 'act-linkedin-email',
		deadline: '2026-07-28'
	},
	{
		id: 'next-2',
		label: 'Revue des comptes touchés par les fuites',
		actionId: 'act-review-leaks',
		deadline: '2026-08-01'
	},
	{
		id: 'next-3',
		label: 'Brouillon RGPD — annuaire',
		actionId: 'act-rgpd-directory',
		deadline: '2026-08-05'
	}
];

export const incidents: Incident[] = [
	{
		id: 'inc-linkedin',
		title: 'E-mail professionnel visible sur LinkedIn',
		severity: 'modérée',
		discoveredAt: '2026-07-20',
		what: 'Le profil LinkedIn du dossier « Emploi actuel » affiche l’adresse e-mail professionnelle en clair.',
		why: 'Cette adresse facilite le phishing ciblé, le scraping et la corrélation avec d’autres traces publiques.',
		impact: 'Phishing, usurpation légère, augmentation du volume de spam ciblé.',
		confidence: 'Élevée — observation directe du profil public.',
		nextStep: 'Masquer ou retirer l’e-mail du profil, puis vérifier la page publique.',
		actionIds: ['act-linkedin-email'],
		exposureIds: ['exp-linkedin-email'],
		folderId: 'folder-job',
		dossier: 'Emploi actuel'
	},
	{
		id: 'inc-leaks',
		title: 'Fuites anciennes d’identifiants',
		severity: 'élevée',
		discoveredAt: '2026-07-18',
		what: 'Deux occurrences d’un e-mail personnel dans des bases de fuites datées (2019 et 2021). Résumés uniquement — aucun dump ni secret stocké.',
		why: 'Si un mot de passe a été réutilisé, des comptes encore actifs peuvent être exposés.',
		impact: 'Compromission de comptes, reprise d’accès non autorisée.',
		confidence: 'Moyenne — correspondance e-mail, contexte de fuite partiel.',
		nextStep: 'Passer en revue les services concernés et confirmer rotation MFA / mots de passe (hors MANTIS).',
		actionIds: ['act-review-leaks'],
		exposureIds: ['exp-leak-2019', 'exp-leak-2021'],
		folderId: 'folder-perso',
		dossier: 'Personnel'
	},
	{
		id: 'inc-directory',
		title: 'Ancienne adresse dans un annuaire public',
		severity: 'faible',
		discoveredAt: '2026-07-15',
		what: 'Un annuaire web liste encore une ancienne adresse postale liée au nom de l’utilisateur.',
		why: 'Donnée personnelle inutilement publique ; peut servir à du doxing ou du phishing de récupération.',
		impact: 'Exposition de données personnelles, harcèlement potentiel.',
		confidence: 'Élevée.',
		nextStep: 'Préparer une demande d’effacement RGPD vers l’éditeur de l’annuaire.',
		actionIds: ['act-rgpd-directory'],
		exposureIds: ['exp-directory'],
		rgpdId: 'rgpd-effacement-1',
		folderId: 'folder-perso',
		dossier: 'Personnel'
	}
];

export const actions: ActionItem[] = [
	{
		id: 'act-linkedin-email',
		title: 'Masquer l’e-mail sur LinkedIn',
		priority: 'haute',
		difficulty: 'facile',
		deadline: '2026-07-28',
		status: 'a_faire',
		guidance: [
			'Ouvrir les paramètres de confidentialité du profil LinkedIn.',
			'Retirer ou masquer l’adresse e-mail professionnelle de la vue publique.',
			'Recharger le profil en navigation privée pour vérifier.',
			'Revenir ici et marquer l’action comme faite.'
		],
		proofExpected: 'Capture ou note : e-mail plus visible en mode public.',
		incidentId: 'inc-linkedin',
		folderId: 'folder-job',
		dossier: 'Emploi actuel'
	},
	{
		id: 'act-review-leaks',
		title: 'Revue des comptes liés aux fuites',
		priority: 'critique',
		difficulty: 'moyenne',
		deadline: '2026-08-01',
		status: 'en_cours',
		guidance: [
			'Lister les services où l’e-mail fuité était utilisé (hors MANTIS).',
			'Pour chaque compte encore actif : changer le mot de passe et activer MFA.',
			'Ne jamais saisir de mots de passe dans MANTIS.',
			'Noter uniquement les services traités (preuve de revue).'
		],
		proofExpected: 'Liste des services revus (noms uniquement).',
		incidentId: 'inc-leaks',
		folderId: 'folder-perso',
		dossier: 'Personnel'
	},
	{
		id: 'act-rgpd-directory',
		title: 'Demande d’effacement — annuaire',
		priority: 'moyenne',
		difficulty: 'facile',
		deadline: '2026-08-05',
		status: 'a_faire',
		guidance: [
			'Ouvrir le module DPO avec le brouillon préparé.',
			'Vérifier le destinataire et le résumé des données.',
			'Copier le brouillon et l’envoyer via votre canal (mail / formulaire).',
			'Enregistrer la date d’envoi dans le suivi DPO (manuel pour l’instant).'
		],
		proofExpected: 'Accusé d’envoi ou capture du formulaire soumis.',
		incidentId: 'inc-directory',
		rgpdId: 'rgpd-effacement-1',
		folderId: 'folder-perso',
		dossier: 'Personnel'
	}
];

export const rgpdRequests: RgpdRequest[] = [
	{
		id: 'rgpd-effacement-1',
		type: 'effacement',
		target: 'Annuaire Web Exemple',
		dpoContact: 'privacy@annuaire-exemple.example',
		status: 'prete',
		identities: ['Nom complet', 'Ancienne adresse postale'],
		dataSummary: 'Adresse postale ancienne publiée sur une fiche nominative.',
		draftPreview: `Objet : Demande d'effacement — article 17 RGPD

Madame, Monsieur,

Je souhaite exercer mon droit à l'effacement concernant les données personnelles me concernant publiées sur votre annuaire (nom et ancienne adresse postale).

Je vous prie de supprimer ces informations dans les meilleurs délais et de me confirmer la suppression.

Cordialement`,
		createdAt: '2026-07-22',
		incidentId: 'inc-directory',
		actionId: 'act-rgpd-directory'
	},
	{
		id: 'rgpd-acces-1',
		type: 'acces',
		target: 'Service newsletter (exemple)',
		dpoContact: 'dpo@newsletter-exemple.example',
		status: 'brouillon',
		identities: ['e-mail personnel'],
		dataSummary: 'Données de contact et historique d’inscription newsletter.',
		draftPreview: `Objet : Demande d'accès — article 15 RGPD

Madame, Monsieur,

Je souhaite obtenir une copie des données personnelles me concernant détenues par votre service, ainsi que les finalités et destinataires éventuels.

Cordialement`,
		createdAt: '2026-07-21'
	}
];

export const alertLevelLabel: Record<AlertLevel, string> = {
	info: 'Information',
	low: 'Faible',
	moderate: 'Modérée',
	high: 'Élevée',
	critical: 'Critique'
};

export const rgpdTypeLabel: Record<RgpdType, string> = {
	acces: 'Accès',
	rectification: 'Rectification',
	effacement: 'Effacement',
	opposition: 'Opposition',
	dereferencement: 'Déréférencement'
};

export const actionStatusLabel: Record<ActionStatus, string> = {
	a_faire: 'À faire',
	en_cours: 'En cours',
	faite: 'Faite'
};

export const identityKindLabel: Record<IdentityKind, string> = {
	nom: 'Nom',
	email: 'E-mail',
	telephone: 'Téléphone',
	pseudo: 'Pseudo',
	domaine: 'Domaine',
	url: 'URL / profil'
};

export const exposureKindLabel: Record<ExposureKind, string> = {
	profil_public: 'Profil public',
	fuite: 'Fuite',
	annuaire: 'Annuaire',
	mention: 'Mention'
};

export const exposureStatusLabel: Record<ExposureStatus, string> = {
	nouvelle: 'Nouvelle',
	en_suivi: 'En suivi',
	acceptee: 'Acceptée',
	reduite: 'Réduite'
};

export function getFolder(id: string): Folder | undefined {
	return folders.find((f) => f.id === id);
}

export function getIdentity(id: string): Identity | undefined {
	return identities.find((i) => i.id === id);
}

export function getExposure(id: string): Exposure | undefined {
	return exposures.find((e) => e.id === id);
}

export function getIncident(id: string): Incident | undefined {
	return incidents.find((i) => i.id === id);
}

export function getAction(id: string): ActionItem | undefined {
	return actions.find((a) => a.id === id);
}

export function getRgpd(id: string): RgpdRequest | undefined {
	return rgpdRequests.find((r) => r.id === id);
}

export function exposuresForFolder(folderId: string): Exposure[] {
	return exposures.filter((e) => e.folderId === folderId);
}

export function exposuresForIdentity(identityId: string): Exposure[] {
	return exposures.filter((e) => e.identityIds.includes(identityId));
}

export function identitiesForFolder(folderId: string): Identity[] {
	return identities.filter((i) => i.folderId === folderId);
}

export function incidentsForFolder(folderId: string): Incident[] {
	return incidents.filter((i) => i.folderId === folderId);
}

/** Remediation guides (pedagogical, Phase 7 preview in Phase 1 UI) */
export interface Guide {
	id: string;
	title: string;
	summary: string;
	when: string;
	steps: string[];
	relatedActionId?: string;
	relatedHref?: string;
}

export const guides: Guide[] = [
	{
		id: 'guide-mfa',
		title: 'Activer la MFA sans stocker de secrets',
		summary: 'Renforcez vos comptes critiques. MANTIS ne demande jamais vos codes.',
		when: 'Après une fuite d’e-mail, ou pour tout compte important (mail, banque, travail).',
		steps: [
			'Ouvrir les paramètres de sécurité du service concerné (hors MANTIS).',
			'Activer une MFA (application d’authentification de préférence).',
			'Vérifier les méthodes de récupération (pas de SMS seul si possible).',
			'Noter dans Actions que la revue du compte est faite — sans saisir le secret.'
		],
		relatedActionId: 'act-review-leaks'
	},
	{
		id: 'guide-privacy-social',
		title: 'Réduire ce que les profils publics affichent',
		summary: 'Moins de données visibles = moins de phishing ciblé.',
		when: 'Quand une exposition de type profil public apparaît.',
		steps: [
			'Ouvrir le profil en navigation privée pour voir la vue « publique ».',
			'Masquer e-mail, téléphone, adresse si visibles.',
			'Recontrôler après 24 h.',
			'Revenir sur l’action liée et la marquer faite.'
		],
		relatedActionId: 'act-linkedin-email'
	},
	{
		id: 'guide-rgpd',
		title: 'Préparer une demande RGPD sans stress',
		summary: 'MANTIS rédige ; vous envoyez. C’est voulu.',
		when: 'Quand une donnée inutile reste publique chez un tiers.',
		steps: [
			'Ouvrir DPO / RGPD et choisir la démarche.',
			'Vérifier la cible et le contact (lecture seule pour l’instant).',
			'Copier le brouillon et l’envoyer via votre canal.',
			'Marquer « J’ai envoyé » pour le suivi.'
		],
		relatedHref: '/dpo'
	}
];

/** Watch / routine stubs (Phase 7 preview) */
export interface WatchRoutine {
	id: string;
	title: string;
	frequency: string;
	sources: string;
	transmitted: string;
	lastRun: string;
	nextRun: string;
	status: 'planifiee' | 'ok' | 'erreur' | 'desactivee';
	note: string;
}

export const watchRoutines: WatchRoutine[] = [
	{
		id: 'watch-breaches',
		title: 'Vérification fuites (e-mails du dossier Personnel)',
		frequency: 'Hebdomadaire',
		sources: 'API breach intelligence (mock — non appelée)',
		transmitted: 'Hash / e-mail uniquement (principe) — rien n’est envoyé en Phase 1',
		lastRun: 'Jamais (mock)',
		nextRun: 'Après branchement OSINT (Phase 4)',
		status: 'planifiee',
		note: 'Aucune requête réseau réelle pour l’instant.'
	},
	{
		id: 'watch-search',
		title: 'Mentions publiques (nom + pseudo)',
		frequency: 'Mensuelle',
		sources: 'Moteur de recherche (mock)',
		transmitted: 'Requêtes ciblées sur vos identités autorisées',
		lastRun: 'Jamais (mock)',
		nextRun: 'Phase 4',
		status: 'planifiee',
		note: 'Limité à vos dossiers — pas de ciblage de tiers.'
	},
	{
		id: 'watch-pages',
		title: 'Pages que vous avez ajoutées',
		frequency: 'Sur demande',
		sources: 'URLs que vous surveillez',
		transmitted: 'URL publique uniquement',
		lastRun: '—',
		nextRun: 'Quand vous ajouterez une page (Phase 4)',
		status: 'desactivee',
		note: 'Ajout de pages non disponible avant la collecte OSINT.'
	}
];

export const watchStatusLabel: Record<WatchRoutine['status'], string> = {
	planifiee: 'Planifiée',
	ok: 'OK',
	erreur: 'Erreur',
	desactivee: 'Désactivée'
};

/** Graph edges for relationship map (no canvas lib yet) */
export interface GraphEdge {
	id: string;
	fromLabel: string;
	fromHref: string;
	relation: string;
	toLabel: string;
	toHref: string;
}

export const graphEdges: GraphEdge[] = [
	{
		id: 'ge-1',
		fromLabel: 'E-mail pro',
		fromHref: '/identites?id=id-email-pro',
		relation: 'apparaît dans',
		toLabel: 'Exposition LinkedIn',
		toHref: '/expositions?id=exp-linkedin-email'
	},
	{
		id: 'ge-2',
		fromLabel: 'Exposition LinkedIn',
		fromHref: '/expositions?id=exp-linkedin-email',
		relation: 'ouvre',
		toLabel: 'Incident e-mail pro',
		toHref: '/incidents?id=inc-linkedin'
	},
	{
		id: 'ge-3',
		fromLabel: 'Incident e-mail pro',
		fromHref: '/incidents?id=inc-linkedin',
		relation: 'recommande',
		toLabel: 'Action masquer e-mail',
		toHref: '/actions?id=act-linkedin-email'
	},
	{
		id: 'ge-4',
		fromLabel: 'E-mail personnel',
		fromHref: '/identites?id=id-email-perso',
		relation: 'lié à',
		toLabel: 'Fuites 2019 / 2021',
		toHref: '/incidents?id=inc-leaks'
	},
	{
		id: 'ge-5',
		fromLabel: 'Incident annuaire',
		fromHref: '/incidents?id=inc-directory',
		relation: 'prépare',
		toLabel: 'Demande RGPD effacement',
		toHref: '/dpo?id=rgpd-effacement-1'
	}
];

export const reportSnapshot = {
	title: 'Aperçu de rapport de posture',
	generatedAt: '2026-07-25 (mock)',
	score: postureScore.value,
	openIncidents: incidents.length,
	openActions: actions.filter((a) => a.status !== 'faite').length,
	rgpdInProgress: rgpdRequests.filter((r) => r.status !== 'repondue').length,
	highlights: [
		'3 incidents ouverts, dont 1 de gravité élevée (fuites anciennes).',
		'1 démarche RGPD prête à envoyer (annuaire).',
		'Priorité n°1 : masquer l’e-mail professionnel sur LinkedIn.'
	],
	note: 'Export HTML/PDF réel prévu en Phase 7. Cet aperçu est local et fictif.'
};

export const appPrinciples = [
	{
		title: 'Local-first',
		text: 'Vos données restent sur cette machine. Pas de cloud obligatoire.'
	},
	{
		title: 'Pas de télémétrie',
		text: 'Aucune collecte automatique d’usage vers un serveur externe.'
	},
	{
		title: 'Pas de secrets',
		text: 'Mots de passe, MFA, seeds et dumps de fuites ne sont jamais stockés.'
	},
	{
		title: 'RGPD assisté',
		text: 'MANTIS prépare ; vous envoyez. Aucun envoi automatique.'
	}
];

export function getGuide(id: string): Guide | undefined {
	return guides.find((g) => g.id === id);
}

/** CSS tone class for severity / priority badges */
export function toneClass(
	value: Severity | Priority | Difficulty
): 'tone-danger' | 'tone-warn' | 'tone-ok' | '' {
	if (value === 'critique' || value === 'élevée' || value === 'haute' || value === 'difficile') {
		return 'tone-danger';
	}
	if (value === 'modérée' || value === 'moyenne') {
		return 'tone-warn';
	}
	if (value === 'faible' || value === 'basse' || value === 'facile') {
		return 'tone-ok';
	}
	return '';
}
