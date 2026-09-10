# lingua-sync — compte Cymbra ID et synchronisation multi-appareils

## ADDED Requirements

### Requirement: Compte optionnel, local-first préservé
L'extension et l'app conteneur SHALL rester pleinement utilisables sans compte : surlignage, statuts, decks, révision et stats locales fonctionnent sans connexion ni sign-in, et la synchronisation ne SHALL démarrer qu'après une connexion explicite (opt-in). Une déconnexion SHALL arrêter la synchronisation sans altérer l'état local.

#### Scenario: Usage sans compte inchangé
- **WHEN** un utilisateur sans compte lit une page avec l'extension installée
- **THEN** le surlignage, le popup de mot et la révision fonctionnent intégralement et aucune requête vers le backend Cymbra n'est émise

#### Scenario: Déconnexion sans perte
- **WHEN** un utilisateur connecté se déconnecte de l'extension
- **THEN** ses statuts, cartes et stats locaux restent intacts et utilisables hors ligne, et plus aucune requête de sync n'est émise

### Requirement: Audience `lingua` admise par configuration seule
Les jetons Lingua SHALL être émis et rafraîchis sous l'audience `lingua`, admise en ajoutant `lingua` à la liste de configuration `CYMBRA_ALLOWED_AUDIENCES` — sans nouveau code d'identité, sans nouveau rôle et sans nouveau scope (`SCOPES`/`APP_SCOPES` du socle inchangés).

#### Scenario: Émission d'un jeton lingua
- **WHEN** un client appelle `SignInLocal` ou `SignInOidc` avec l'audience `lingua` sur un serveur dont `CYMBRA_ALLOWED_AUDIENCES` contient `lingua`
- **THEN** un `TokenPair` est émis avec l'audience `lingua` et le refresh fonctionne pour cette audience

#### Scenario: Audience non configurée refusée
- **WHEN** un client demande l'audience `lingua` sur un serveur dont la configuration ne la liste pas
- **THEN** la connexion est refusée par le contrôle d'audience existant

### Requirement: Connexion dans l'extension en bearer gRPC-web
L'extension SHALL se connecter en gRPC-web avec un `TokenPair` bearer (jamais la surface cookie `/web/auth`) : intercepteurs auth/refresh sur le modèle du back-office, refresh single-flight avec un seul retry, access token conservé en `storage.session` et refresh token en `storage.local`. Deux méthodes SHALL être offertes : OIDC Google via `chrome.identity.launchWebAuthFlow` (client id ajouté au CSV `CYMBRA_GOOGLE_AUDIENCE`) et email/mot de passe (`SignInLocal`).

#### Scenario: Connexion Google depuis l'extension
- **WHEN** l'utilisateur choisit « Continuer avec Google » dans l'extension
- **THEN** le flow `launchWebAuthFlow` produit un id_token, `SignInOidc` retourne un `TokenPair` d'audience `lingua`, et les appels suivants portent `Authorization: Bearer`

#### Scenario: Access token expiré rafraîchi silencieusement
- **WHEN** un appel de sync échoue en UNAUTHENTICATED alors qu'un refresh token valide est présent
- **THEN** le client rafraîchit une seule fois (single-flight), rejoue l'appel avec le nouveau jeton, et l'utilisateur ne voit aucune interruption

#### Scenario: Redémarrage du navigateur
- **WHEN** l'utilisateur redémarre son navigateur
- **THEN** l'access token de `storage.session` a disparu, le refresh token de `storage.local` obtient un nouveau `TokenPair`, et la session continue sans re-saisie

### Requirement: Origines gRPC-web générales sans élargir la liste console
Le serveur SHALL admettre les origines navigateur produit sur la surface gRPC-web via une nouvelle liste `CYMBRA_ALLOWED_WEB_ORIGINS` (dont `chrome-extension://<id>`), servie en union avec `CYMBRA_BACK_OFFICE_ORIGINS` par la couche CORS de tonic ; la liste console ne SHALL PAS être élargie et la nouvelle liste ne SHALL PAS être credentialed (bearer uniquement, pas de cookies). Une liste vide SHALL rester le défaut (aucune origine produit admise).

#### Scenario: Preflight de l'extension accepté
- **WHEN** l'extension publiée émet un appel gRPC-web et son origine `chrome-extension://<id>` figure dans `CYMBRA_ALLOWED_WEB_ORIGINS`
- **THEN** le preflight CORS et l'appel aboutissent, sans que cette origine apparaisse dans `CYMBRA_BACK_OFFICE_ORIGINS`

#### Scenario: Origine inconnue bloquée
- **WHEN** une page web d'origine non listée tente un appel gRPC-web
- **THEN** le navigateur bloque l'appel par CORS, et l'intercepteur d'auth reste l'autorité d'autorisation pour tout appel qui parvient au serveur

### Requirement: Connexion native dans l'app Apple avec Sign in with Apple
L'app conteneur SHALL se connecter nativement (tonic natif, `SignInOidc`/`SignInLocal`, audience `lingua`, jetons dans le Keychain) et, dès lors qu'un login tiers y est proposé sur iOS, SHALL proposer Sign in with Apple (l'OIDC Apple du backend existant) au même niveau. L'extension Safari SHALL consommer la session de l'app (un seul compte par appareil Apple) sans flow OAuth dans Safari.

#### Scenario: Sign in with Apple présent
- **WHEN** l'écran de connexion iOS affiche « Continuer avec Google »
- **THEN** « Continuer avec Apple » est proposé au même niveau et aboutit à un `TokenPair` d'audience `lingua` via l'OIDC Apple existant

#### Scenario: L'extension Safari hérite de la session
- **WHEN** l'utilisateur est connecté dans l'app conteneur et lit une page dans Safari
- **THEN** la sync de l'extension opère sous le compte de l'app, sans écran de connexion dans Safari

### Requirement: Module backend isolé et inerte sans configuration
Le backend Lingua SHALL être un crate `backend/lingua` calqué sur `backend/music` : schéma Postgres `lingua` possédé par un rôle `lingua_svc` à `search_path` épinglé, migrations propres, `UserPort` injecté pour tout besoin compte (jamais de lecture d'un autre schéma), protos `cymbra.lingua.v1` dans `backend/lingua/proto`. Sans `CYMBRA_LINGUA_DATABASE_URL`, les services Lingua ne SHALL PAS être câblés et le serveur SHALL démarrer normalement.

#### Scenario: Serveur sans base lingua
- **WHEN** le serveur démarre sans `CYMBRA_LINGUA_DATABASE_URL`
- **THEN** il sert normalement les autres modules et journalise que les services Lingua sont désactivés

#### Scenario: Isolation de schéma
- **WHEN** une requête du module Lingua s'exécute
- **THEN** elle tourne sur le pool `lingua_svc`, dont le `search_path` résout uniquement le schéma `lingua`

### Requirement: Synchronisation des statuts par op-log et last-write-wins
`KnownWordsService` SHALL synchroniser les statuts par (langue, lemme) : le client pousse une outbox d'opérations horodatées (lots idempotents, reprise par offset), le serveur résout en last-write-wins par lemme (horodatage le plus récent, tie-break déterministe par appareil), et le client tire les changements par curseur delta ; l'amorçage ou un curseur invalide SHALL passer par un snapshot gardé par ETag/version (réponse « inchangé » sans corps si l'ETag correspond).

#### Scenario: Statut propagé entre deux appareils
- **WHEN** l'utilisateur marque `seldom` connu sur son Mac puis synchronise son iPhone
- **THEN** l'iPhone reçoit le statut `known` de `seldom` via le pull par curseur

#### Scenario: Conflit résolu par le geste le plus récent
- **WHEN** deux appareils hors ligne posent des statuts différents sur le même lemme puis synchronisent
- **THEN** le statut à l'horodatage le plus récent gagne sur le serveur et les deux appareils convergent vers lui

#### Scenario: Push interrompu repris sans doublon
- **WHEN** un push d'outbox est interrompu puis rejoué intégralement
- **THEN** l'état serveur est identique à celui d'un push unique (idempotence par lot)

### Requirement: Synchronisation des cartes complètes, médias exclus
`DeckService` SHALL synchroniser les cartes complètes — lemme, forme vue, phrase de provenance, source explicitement capturée, glose, état FSRS — identifiées par un id client stable, en last-write-wins par carte (suppressions incluses). Le contenu du champ `media` ne SHALL PAS être synchronisé dans ce change (l'emplacement du schéma reste local).

#### Scenario: Carte créée sur mobile, révisée sur desktop
- **WHEN** une carte créée en lisant dans Safari iOS est synchronisée puis l'utilisateur ouvre le side panel sur son Mac
- **THEN** la carte y apparaît avec sa phrase de provenance et son état FSRS, et sa révision sur le Mac se propage en retour

#### Scenario: Carte avec image
- **WHEN** une carte locale porte un média et est synchronisée
- **THEN** tous ses champs montent sauf le contenu du média, qui reste sur l'appareil d'origine

### Requirement: Fusion du store local au premier sign-in
À la première connexion d'un appareil ayant un état local pré-compte, le client SHALL pousser cet état intégralement comme opérations conservant leurs horodatages d'origine, puis tirer l'état fusionné ; la fusion SHALL être le last-write-wins ordinaire du protocole (aucun cas spécial) et ne SHALL rien perdre : tout statut, carte ou agrégat local absent du serveur est créé.

#### Scenario: Premier sign-in après des mois d'usage local
- **WHEN** un utilisateur avec 2 000 statuts et 150 cartes locaux se connecte pour la première fois
- **THEN** l'intégralité monte avec les horodatages d'origine, et l'état fusionné redescendu contient l'union avec l'éventuel état serveur préexistant

#### Scenario: Deuxième appareil déjà utilisé localement
- **WHEN** un second appareil avec son propre état local se connecte au même compte
- **THEN** les deux états sont fusionnés par last-write-wins et aucune carte d'aucun des deux appareils n'est perdue

### Requirement: Allow-list stricte de ce qui monte au serveur
Seules trois familles de données SHALL monter : statuts de lemmes, cartes, agrégats de stats. Aucune URL de navigation, aucun texte de page, aucun historique de lecture web ne SHALL être transmis ni stocké serveur — la seule source montante est celle portée par une carte explicitement créée par l'utilisateur. Les compteurs d'exposition du MVP SHALL rester locaux dans ce change.

#### Scenario: Lecture sans capture
- **WHEN** un utilisateur connecté lit dix pages sans créer de carte ni toucher un statut
- **THEN** aucune donnée relative à ces pages (URL, texte, comptes par page) n'est transmise au serveur

#### Scenario: La carte est la seule exception
- **WHEN** l'utilisateur crée une carte depuis une page
- **THEN** la phrase et la source de cette carte montent avec elle, et rien d'autre de la page ne monte

### Requirement: Le plugin Claude Code reste hors synchronisation
Le store local du plugin Claude Code (`~/.lingua/`) ne SHALL acquérir aucun chemin réseau dans ce change : l'invariant « aucune connexion réseau » de l'ingestion du MVP SHALL rester testé et vrai, la synchronisation des données du plugin étant un change ultérieur.

#### Scenario: Ingestion toujours hors ligne
- **WHEN** le hook `Stop` ingère un transcript alors que l'utilisateur possède un compte Lingua connecté dans son extension
- **THEN** l'ingestion n'ouvre aucune connexion réseau et le store du plugin reste purement local

### Requirement: Purge des données Lingua à la suppression du compte
La suppression du compte (`DeleteAccount`) SHALL purger toutes les données `lingua.*` de l'utilisateur via le job `purge_user` existant, étendu (handler worker + `search_path` du rôle admin incluant `lingua`), de manière idempotente ; un compte sans données Lingua SHALL être un no-op pour cette étape.

#### Scenario: Suppression d'un compte avec données Lingua
- **WHEN** un utilisateur avec statuts, cartes et stats serveur supprime son compte
- **THEN** le job `purge_user` efface toutes ses lignes des tables du schéma `lingua`

#### Scenario: Purge rejouée
- **WHEN** le job `purge_user` est rejoué pour le même utilisateur
- **THEN** il réussit sans erreur et sans effet supplémentaire

### Requirement: Socle consommé sans redéclaration
Lingua SHALL consommer le socle tel quel : l'évaluation des feature flags SHALL dériver l'app de l'audience du jeton (`lingua` automatique, aucun nouveau mécanisme), les évènements d'usage SHALL passer par le service analytics existant (`platform` = `web` pour l'extension, `ios`/`macos` pour l'app), et les services `cymbra.lingua.v1` SHALL être atteignables par la route gRPC/gRPC-web existante sans modification du reverse proxy (aucune nouvelle route HTTP).

#### Scenario: Flag évalué pour l'audience lingua
- **WHEN** un client connecté avec un jeton `lingua` évalue un flag scopé à l'app `lingua`
- **THEN** le flag est évalué dans le contexte `lingua` sans configuration supplémentaire côté flags

#### Scenario: Appel gRPC-web routé sans changement Caddy
- **WHEN** l'extension appelle `/cymbra.lingua.v1.KnownWordsService/…` à travers le reverse proxy de production
- **THEN** l'appel atteint tonic par la branche par défaut existante, preflight CORS compris
