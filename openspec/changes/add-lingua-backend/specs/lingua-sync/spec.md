# lingua-sync — compte Cymbra ID et synchronisation : serveur et protocole

## ADDED Requirements

### Requirement: Audience `lingua` admise par configuration seule
Les jetons Lingua SHALL être émis et rafraîchis sous l'audience `lingua`, admise en ajoutant `lingua` à la liste de configuration `CYMBRA_ALLOWED_AUDIENCES` — sans nouveau code d'identité, sans nouveau rôle et sans nouveau scope (`SCOPES`/`APP_SCOPES` du socle inchangés).

#### Scenario: Émission d'un jeton lingua
- **WHEN** un client appelle `SignInLocal` ou `SignInOidc` avec l'audience `lingua` sur un serveur dont `CYMBRA_ALLOWED_AUDIENCES` contient `lingua`
- **THEN** un `TokenPair` est émis avec l'audience `lingua` et le refresh fonctionne pour cette audience

#### Scenario: Audience non configurée refusée
- **WHEN** un client demande l'audience `lingua` sur un serveur dont la configuration ne la liste pas
- **THEN** la connexion est refusée par le contrôle d'audience existant

### Requirement: Origines gRPC-web générales sans élargir la liste console
Le serveur SHALL admettre les origines navigateur produit sur la surface gRPC-web via une nouvelle liste `CYMBRA_ALLOWED_WEB_ORIGINS` (dont `chrome-extension://<id>`), servie en union avec `CYMBRA_BACK_OFFICE_ORIGINS` par la couche CORS de tonic ; la liste console ne SHALL PAS être élargie et la nouvelle liste ne SHALL PAS être credentialed (bearer uniquement, pas de cookies). Une liste vide SHALL rester le défaut (aucune origine produit admise).

#### Scenario: Preflight de l'extension accepté
- **WHEN** l'extension publiée émet un appel gRPC-web et son origine `chrome-extension://<id>` figure dans `CYMBRA_ALLOWED_WEB_ORIGINS`
- **THEN** le preflight CORS et l'appel aboutissent, sans que cette origine apparaisse dans `CYMBRA_BACK_OFFICE_ORIGINS`

#### Scenario: Origine inconnue bloquée
- **WHEN** une page web d'origine non listée tente un appel gRPC-web
- **THEN** le navigateur bloque l'appel par CORS, et l'intercepteur d'auth reste l'autorité d'autorisation pour tout appel qui parvient au serveur

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

### Requirement: Allow-list stricte de ce qui monte au serveur
Seules trois familles de données SHALL monter : statuts de lemmes, cartes, agrégats de stats. Aucune URL de navigation, aucun texte de page, aucun historique de lecture web ne SHALL être transmis ni stocké serveur — la seule source montante est celle portée par une carte explicitement créée par l'utilisateur. Les compteurs d'exposition de la pile locale SHALL rester locaux dans ce change.

#### Scenario: Lecture sans capture
- **WHEN** un utilisateur connecté lit dix pages sans créer de carte ni toucher un statut
- **THEN** aucune donnée relative à ces pages (URL, texte, comptes par page) n'est transmise au serveur

#### Scenario: La carte est la seule exception
- **WHEN** l'utilisateur crée une carte depuis une page
- **THEN** la phrase et la source de cette carte montent avec elle, et rien d'autre de la page ne monte

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
