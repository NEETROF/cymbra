# lingua-sync — compte Cymbra ID et synchronisation : clients

## ADDED Requirements

### Requirement: Compte optionnel, local-first préservé
L'extension et l'app conteneur SHALL rester pleinement utilisables sans compte : surlignage, statuts, decks, révision et stats locales fonctionnent sans connexion ni sign-in, et la synchronisation ne SHALL démarrer qu'après une connexion explicite (opt-in). Une déconnexion SHALL arrêter la synchronisation sans altérer l'état local.

#### Scenario: Usage sans compte inchangé
- **WHEN** un utilisateur sans compte lit une page avec l'extension installée
- **THEN** le surlignage, le popup de mot et la révision fonctionnent intégralement et aucune requête vers le backend Cymbra n'est émise

#### Scenario: Déconnexion sans perte
- **WHEN** un utilisateur connecté se déconnecte de l'extension
- **THEN** ses statuts, cartes et stats locaux restent intacts et utilisables hors ligne, et plus aucune requête de sync n'est émise

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

### Requirement: Connexion native dans l'app Apple avec Sign in with Apple
L'app conteneur SHALL se connecter nativement (tonic natif, `SignInOidc`/`SignInLocal`, audience `lingua`, jetons dans le Keychain) et, dès lors qu'un login tiers y est proposé sur iOS, SHALL proposer Sign in with Apple (l'OIDC Apple du backend existant) au même niveau. L'extension Safari SHALL consommer la session de l'app (un seul compte par appareil Apple) sans flow OAuth dans Safari.

#### Scenario: Sign in with Apple présent
- **WHEN** l'écran de connexion iOS affiche « Continuer avec Google »
- **THEN** « Continuer avec Apple » est proposé au même niveau et aboutit à un `TokenPair` d'audience `lingua` via l'OIDC Apple existant

#### Scenario: L'extension Safari hérite de la session
- **WHEN** l'utilisateur est connecté dans l'app conteneur et lit une page dans Safari
- **THEN** la sync de l'extension opère sous le compte de l'app, sans écran de connexion dans Safari

### Requirement: Fusion du store local au premier sign-in
À la première connexion d'un appareil ayant un état local pré-compte, le client SHALL pousser cet état intégralement comme opérations conservant leurs horodatages d'origine, puis tirer l'état fusionné ; la fusion SHALL être le last-write-wins ordinaire du protocole (aucun cas spécial) et ne SHALL rien perdre : tout statut, carte ou agrégat local absent du serveur est créé.

#### Scenario: Premier sign-in après des mois d'usage local
- **WHEN** un utilisateur avec 2 000 statuts et 150 cartes locaux se connecte pour la première fois
- **THEN** l'intégralité monte avec les horodatages d'origine, et l'état fusionné redescendu contient l'union avec l'éventuel état serveur préexistant

#### Scenario: Deuxième appareil déjà utilisé localement
- **WHEN** un second appareil avec son propre état local se connecte au même compte
- **THEN** les deux états sont fusionnés par last-write-wins et aucune carte d'aucun des deux appareils n'est perdue

### Requirement: Le plugin Claude Code reste hors synchronisation
Le store local du plugin Claude Code (`~/.lingua/`) ne SHALL acquérir aucun chemin réseau dans ce change : l'invariant « aucune connexion réseau » de l'ingestion SHALL rester testé et vrai, la synchronisation des données du plugin étant un change ultérieur.

#### Scenario: Ingestion toujours hors ligne
- **WHEN** le hook `Stop` ingère un transcript alors que l'utilisateur possède un compte Lingua connecté dans son extension
- **THEN** l'ingestion n'ouvre aucune connexion réseau et le store du plugin reste purement local
