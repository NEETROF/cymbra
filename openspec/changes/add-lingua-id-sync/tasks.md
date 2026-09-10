# Tasks — add-lingua-id-sync

## 1. Backend — audience et configuration

- [ ] 1.1 Ajouter `lingua` à `CYMBRA_ALLOWED_AUDIENCES` dans `backend/.env.example` et `backend/deploy/.env.prod.example` (+ commentaire) ; test d'intégration : `SignInLocal`/`Refresh` d'audience `lingua` acceptés, audience absente de la conf refusée
- [ ] 1.2 Créer le client OAuth Google de l'extension (type Web, redirect `https://<ext-id>.chromiumapp.org/`) et l'ajouter au CSV `CYMBRA_GOOGLE_AUDIENCE` (env example + doc) — précédent : client desktop
- [ ] 1.3 Introduire `CYMBRA_ALLOWED_WEB_ORIGINS` (config serveur, CSV, défaut vide) et servir l'union `back_office_origins ∪ allowed_web_origins` dans la couche CORS gRPC-web de tonic (`backend/server/src/main.rs`) ; tests : origine de la nouvelle liste admise, origine inconnue refusée, `CYMBRA_BACK_OFFICE_ORIGINS` non élargie
- [ ] 1.4 Vérifier sur pièces que `SCOPES`/`APP_SCOPES` (`backend/platform/src/lib.rs`) ne référencent nulle part `lingua` après ce change (aucun scope ajouté — revue, pas de code)

## 2. Backend — crate `backend/lingua`

- [ ] 2.1 Scaffold `backend/lingua` (crate `cymbra-lingua`) calqué sur `backend/music` : `Cargo.toml` (workspace), `build.rs` (`build_client(false)`), layout `src/` avec seam `*_core.rs` / `pg*.rs` / `grpc.rs`
- [ ] 2.2 Schéma + rôle : entrée `lingua_svc` (schéma `lingua`, `search_path` épinglé) dans `backend/db/init/roles.sql.tpl`, `CYMBRA_LINGUA_DATABASE_URL` dans les env examples, `backend/deploy/provision-lingua-role.sql` + branchement dans `provision-optional-modules.sh` (pattern music)
- [ ] 2.3 Migrations : tables statuts (user, langue, lemme, statut, provenance, horodatage, device, séquence de changement), cartes (id client, champs du schéma MVP hors contenu média, séquence), agrégats stats (user, jour, langue, device) ; index sur (user, séquence) pour le pull par curseur
- [ ] 2.4 Câblage serveur : module inerte sans `CYMBRA_LINGUA_DATABASE_URL` (log « lingua services disabled »), pool propre + MIGRATOR, `UserPort` injecté, services derrière l'intercepteur d'auth strict avec contrôle d'audience
- [ ] 2.5 Mettre à jour `DEPLOY.md` (activation du module, variables, provision)

## 3. Backend — protos et services `cymbra.lingua.v1`

- [ ] 3.1 `backend/lingua/proto/known_words.proto` : `KnownWordsService` — `PushOps` (lots idempotents horodatés), `PullChanges` (curseur delta), `GetSnapshot` (ETag/version, réponse « inchangé ») ; `buf lint` vert (protos nouveaux : `buf breaking` non concerné)
- [ ] 3.2 Logique de sync host-testée (`known_words_core.rs`) : LWW par (langue, lemme) avec tie-break par device, idempotence des lots, séquence monotone, reprise par offset ; tests de convergence (conflit, replay, interleaving)
- [ ] 3.3 `deck.proto` : `DeckService` — push/pull de cartes complètes par id client, LWW par carte, suppressions propagées, champ média jamais transporté ; `deck_core.rs` + tests
- [ ] 3.4 `stats.proto` : `StatsService` — `UpsertDailyStats` (clé jour/langue/device), `GetStats` (plage de dates, séries consolidées SUM par jour × langue) ; `stats_core.rs` + tests (double appareil, upsert rejoué)
- [ ] 3.5 Adaptateurs `pg_known_words.rs` / `pg_deck.rs` / `pg_stats.rs` + `grpc.rs` (auth : l'utilisateur du jeton, jamais un user_id client) ; ajouter les nouveaux `pg*.rs` au périmètre du regex d'exclusion coverage si le pattern existant ne les couvre pas déjà
- [ ] 3.6 Test d'intégration bout-en-bout : deux clients simulés convergent (statuts + cartes + stats) à travers les trois services

## 4. Backend — purge et vie privée

- [ ] 4.1 Étendre `purge_user_with` (`backend/worker/src/lib.rs`) : effacement `lingua.*` pour l'utilisateur, idempotent, no-op sans données ; étendre le `search_path` du rôle admin (`roles.sql.tpl` ligne admin + doc de migration prod)
- [ ] 4.2 Tests du handler worker : purge avec données, purge rejouée, compte sans données Lingua
- [ ] 4.3 Test-contrat « allow-list » : les messages proto de `cymbra.lingua.v1` ne contiennent aucun champ d'URL de page lue, de texte de page ou d'historique (revue de schéma documentée dans le proto + assertion sur les descripteurs si praticable)

## 5. Extension — compte et session

- [ ] 5.1 Transport gRPC-web Connect-ES dans `apps/lingua-extension` : clone adapté de `apps/back-office/src/lib/transport.ts` + `api.ts` (intercepteurs auth / refresh single-flight / session-expiry, seam de test), URL backend par variable de build
- [ ] 5.2 Stockage des jetons : access en `chrome.storage.session`, refresh en `chrome.storage.local` ; reprise de session au démarrage (refresh → nouveau `TokenPair`) ; tests vitest du cycle
- [ ] 5.3 UI compte (popup d'icône + page de réglages, tokens Cymbra) : « Continuer avec Google » (`chrome.identity.launchWebAuthFlow` → `SignInOidc`), email/mot de passe (`SignInLocal`), état connecté (email masquable), déconnexion (`Logout` + purge des jetons, état local intact)
- [ ] 5.4 Doc dev : origine `chrome-extension://` stable par clé de manifest ; ajout de l'origine dev à `CYMBRA_ALLOWED_WEB_ORIGINS` de l'environnement local uniquement

## 6. Extension — outbox et synchronisation

- [ ] 6.1 Outbox locale (op-log versionné dans `chrome.storage.local`) : chaque mutation de statut/carte enfile une op horodatée (device_id généré à l'install) ; vidage par lots avec reprise par offset
- [ ] 6.2 Pull par curseur + application au store local (LWW côté client symétrique au serveur) ; propagation cross-onglets existante (`storage.onChanged`) déclenchée par les changements tirés
- [ ] 6.3 Fusion du premier sign-in : poussée intégrale de l'état pré-compte (horodatages d'origine conservés), puis pull du snapshot fusionné ; test du scénario « deux appareils avec états locaux disjoints »
- [ ] 6.4 Orchestration d'arrière-plan : sync au réveil du service worker, après un lot de mutations et à l'ouverture du side panel ; indicateur discret d'état de sync + action manuelle dans les réglages ; aucune requête tant que non connecté
- [ ] 6.5 Stats : agrégation locale par (jour, langue) → `UpsertDailyStats` avec device_id ; tests vitest (idempotence du re-push)

## 7. Extension — écran de stats

- [ ] 7.1 Écran de stats (page d'extension, tokens Cymbra) : séries par jour × langue (mots appris, révisions, expositions), plage 7/30/90 jours ; connecté = `GetStats` consolidé (portée « tous les appareils »), sinon agrégats locaux (portée « cet appareil ») ; mention « hors sessions d'agents »
- [ ] 7.2 Lint des chaînes UI étendu aux nouveaux écrans (compte, sync, stats) : aucune occurrence de « lemme » (« forme du dictionnaire », « mots différents »)

## 8. App Apple — connexion native et sync

- [ ] 8.1 Écran de connexion natif (SwiftUI, charte Cymbra) : Sign in with Apple (`ASAuthorizationController` → `SignInOidc` Apple) + « Continuer avec Google » + email/mot de passe, via tonic natif d'audience `lingua` ; jetons en Keychain
- [ ] 8.2 Partage de session app ↔ extension Safari via App Group (un compte par appareil) : l'extension synchronise sous la session de l'app, aucun flow OAuth dans Safari ; trancher jetons-par-handler vs copie App Group (question ouverte du design) et documenter
- [ ] 8.3 Sync app : même outbox/pull/fusion que l'extension (types `lingua-core` partagés) pour statuts, cartes, stats ; premier sign-in fusionne l'état local de l'appareil
- [ ] 8.4 Écran de stats de l'app (mêmes règles que 7.1) ; vérification manuelle croisée : mot marqué sur Mac visible sur iPhone, carte iOS révisable sur desktop, stats consolidées justes
- [ ] 8.5 TestFlight interne avec le backend de dev ; parcours de review App Store re-déroulé (Sign in with Apple présent, privacy labels mis à jour : données de compte + contenu utilisateur synchronisé)

## 9. Gates et finitions

- [ ] 9.1 `cargo fmt --all --check` + `clippy --workspace --all-targets -- -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80` (regex d'exclusion partagé à jour pour les nouveaux adaptateurs)
- [ ] 9.2 `buf lint` sur `backend/lingua/proto` ; vérifier que la lane `proto` (`buf breaking`) surveille `backend/lingua/proto/**` pour les changes suivants
- [ ] 9.3 vitest vert sur `apps/lingua-extension` ; `python3 scripts/check_ci_units.py --list` confirme que toutes les unités touchées restent surveillées
- [ ] 9.4 Env/deploy finalisés : `.env.example`, `.env.prod.example`, `DEPLOY.md`, provision ; répétition du plan de migration (backend inerte → provision → activation → clients)
- [ ] 9.5 `openspec validate add-lingua-id-sync --strict` final + mise à jour des specs si l'implémentation a fait bouger un contrat
