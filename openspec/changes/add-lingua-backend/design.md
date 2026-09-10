# Design — add-lingua-backend

## Context

Côté backend, tout le nécessaire existe : `check_audience` (`backend/auth/src/module.rs:126`) itère `CYMBRA_ALLOWED_AUDIENCES` ; la rotation du refresh token avec détection de réutilisation et révocation de famille est en place (testée : `refresh_rotates_then_reuse_revokes_family`) ; l'OIDC Google **et** Apple sont vérifiés serveur (`CYMBRA_GOOGLE_AUDIENCE`/`CYMBRA_APPLE_AUDIENCE`, CSV d'audiences) ; le pattern « module produit » est établi par `backend/music` (schéma + rôle + `search_path` épinglé + MIGRATOR + inertie sans env) ; le job `purge_user` (`#[sqlxmq::job]`, pool `admin_svc`) porte déjà l'effacement cross-schéma. Le delta est donc étroit et surtout de la **configuration** — c'est le but de ce design : le garder étroit.

Contrainte héritée de la pile locale : chaque surface cliente a un **état local versionné** dont les schémas partagent les types de `lingua-core` (décision de `add-lingua-decks-review` et `add-lingua-apple`) — « pour que la fusion soit mécanique » : le protocole défini ici est cette fusion, les clients s'y brancheront au change `add-lingua-connected-clients` (qui porte les décisions côté client : transport bearer gRPC-web, stockage des jetons, OIDC dans l'extension, Sign in with Apple, fusion du store pré-compte).

## Goals / Non-Goals

**Goals :**
- Un backend Lingua complet et **déployable inerte** : crate, migrations, rôles, protos, services — rien ne change pour personne tant que `CYMBRA_LINGUA_DATABASE_URL` est absente.
- Le serveur ne voit **que** ce que la spec autorise : statuts de lemmes, cartes, agrégats. Jamais l'historique de lecture.
- Zéro nouveau rôle, zéro nouveau scope, zéro route HTTP nouvelle : l'empreinte sur le socle = deux variables d'env et une liste CORS.
- Un protocole de sync convergent testable serveur seul (deux clients simulés convergent).

**Non-Goals :**
- Les clients connectés (UI compte, outbox cliente, fusion du store pré-compte, écran de stats) — change `add-lingua-connected-clients`.
- Sync du plugin Claude Code (`~/.lingua/`) — les transcripts sont confidentiels ; change ultérieur avec son propre design (auth CLI loopback PKCE).
- Médias/images de cartes (v1 : le champ `media` du schéma ne se synchronise pas ; la sync chiffrée viendra avec la capture d'image).
- Console back-office Lingua, rôles/scope `lingua`, modération — rien d'admin dans ce change.
- Résolution de conflits fine (CRDT, historique par champ) — voir D3.

## Decisions

### D1 — Audience `lingua` : une entrée de configuration, zéro rôle
`CYMBRA_ALLOWED_AUDIENCES=music,live,back-office,web,lingua` — c'est tout le delta d'identité. `check_audience` accepte l'audience à l'émission comme au refresh ; les jetons `lingua` traversent l'intercepteur d'auth strict comme ceux de `music`. **`SCOPES`/`APP_SCOPES` (`backend/platform/src/lib.rs`) ne bougent pas** : un scope n'existe que pour porter des rôles d'administration scopés, et ce change n'a aucune surface admin. Alternative rejetée : ajouter `LINGUA_SCOPE` « pour plus tard » — cela traînerait l'agrégation de session back-office et l'admin scope-aware pour un usage nul en v1, et l'audit séparation-des-pouvoirs a montré le coût des scopes déclarés-mais-pas-testés. Le scope arrivera avec la première surface admin Lingua (`add-lingua-back-office`).

### D2 — CORS : `CYMBRA_ALLOWED_WEB_ORIGINS`, l'union plutôt que l'élargissement
Nouvelle variable `CYMBRA_ALLOWED_WEB_ORIGINS` : liste générale d'origines navigateur admises sur la surface gRPC-web **bearer-only**. La couche CORS de tonic (aujourd'hui nourrie de `cfg.back_office_origins` seule) passe à l'union `back_office_origins ∪ allowed_web_origins`. C'est là qu'ira `chrome-extension://<id>` (id stable, dérivé de la clé publiée au store). Alternatives rejetées : élargir `CYMBRA_BACK_OFFICE_ORIGINS` — son nom est un contrat (« console only », dit le commentaire d'env) et mêler une origine produit à la liste de la console admin brouille l'audit ; réutiliser `CYMBRA_WEB_ORIGINS` — elle gouverne la surface **cookie credentialed** (`/web/auth/*` + site), exactement ce qu'une extension ne doit pas toucher. La CORS reste de la défense en profondeur : l'autorisation est l'intercepteur d'auth, pas l'origine.

### D3 — Protocole de sync : op-log client, LWW par entité, pull par curseur — pas de CRDT
Chaque client tient une **outbox** (op-log local des mutations : statut posé, carte créée/modifiée/supprimée) vidée vers le serveur par lots idempotents ; chaque op porte l'horodatage client et le `device_id`. Résolution : **last-write-wins par (langue, lemme)** pour les statuts et **par carte** (id client UUID) pour les decks — horodatage le plus récent gagne, tie-break déterministe par `device_id`. Le pull est un **delta par curseur** (séquence de changement serveur monotone par utilisateur) ; le bootstrap ou un curseur invalide passe par un **snapshot** gardé par ETag/version (pas de re-téléchargement si rien n'a bougé). Pourquoi pas de CRDT : l'état est de type ensemble-qui-croît + compteurs — deux appareils qui posent chacun un statut sur le même lemme dans la même minute est le pire cas réel, et « le dernier geste gagne » est exactement la sémantique attendue par l'utilisateur. Un CRDT par champ coûterait le format, la doc et les tests d'un protocole de recherche pour un conflit qui se résout d'un clic. **La fusion du premier sign-in n'est pas un cas spécial** : le client poussera son état pré-compte comme une grosse outbox aux horodatages d'origine (décision côté client au change suivant) — le serveur applique le LWW ordinaire.

### D4 — Ce que le serveur stocke : trois tables de données, une allow-list
Schéma `lingua` : statuts par (user, langue, lemme, statut, provenance, horodatage, device) ; cartes complètes au schéma de la pile locale **moins** le contenu du champ `media` (l'emplacement reste, rien ne monte) — la phrase de provenance et la source d'une carte montent : ce sont les **données personnelles que l'utilisateur a explicitement capturées**, pas de l'historique ; agrégats de stats (D5). L'allow-list est le contrat de la spec : toute donnée absente de ces trois familles ne monte pas — en particulier aucune URL de page simplement lue, aucun texte de page, aucun compteur par-site. Les compteurs d'exposition de la pile locale restent **locaux** en v1 (volumineux, faible valeur multi-appareils, et c'est la donnée la plus proche de l'historique de lecture — la garder locale est aussi une position produit).

### D5 — Stats : agrégats additifs par (jour, langue, appareil), consolidation à la lecture
Chaque appareil upserte ses lignes d'agrégat `(jour UTC, langue, device_id) → {expositions, mots appris, révisions faites}` — idempotent (upsert par clé), jamais d'événement fin horodaté. La lecture (`StatsService.GetStats`) somme sur les appareils et renvoie des séries par jour × langue, consommées par l'écran de stats des clients (change suivant). Alternative rejetée : compteurs absolus LWW par (jour, langue) — deux appareils actifs le même jour s'écraseraient mutuellement ; la clé par appareil rend l'addition juste par construction. Vie privée : le grain jour × langue est le plus fin autorisé — pas d'heure, pas de source, pas de site.

### D6 — Module backend : `backend/lingua` calqué sur `backend/music`, inerte sans env
Crate `cymbra-lingua` : schéma Postgres `lingua` possédé par le rôle `lingua_svc` à `search_path` épinglé (`roles.sql.tpl` + `provision-lingua-role.sql` pour la prod, pattern `provision-music-role.sql`), MIGRATOR propre, protos `cymbra.lingua.v1` dans `backend/lingua/proto` (`build_client(false)` — pas de transport interne, conformément à la règle des trois objets). Le serveur ne câble les trois services que si `CYMBRA_LINGUA_DATABASE_URL` est posée — sinon `tracing::info!("lingua services disabled")`, comme music. `UserPort` injecté (le trait du crate `user-port`, déclaré côté consommateur) pour l'existence/l'état du compte — jamais de lecture du schéma `user_account`. Logique de merge/curseur en `lingua_sync_core.rs` host-testé ; adaptateurs `pg*.rs`/`grpc.rs` couverts par le regex d'exclusion existant.

### D7 — Purge : étendre `purge_user`, pas créer un job
`DeleteAccount` déclenche déjà le job `purge_user` (worker, pool `admin_svc`, idempotent). Le delta : `purge_user_with` efface aussi `lingua.*` pour l'utilisateur, et le `search_path` du rôle admin (`roles.sql.tpl`) gagne le schéma `lingua`. Un compte sans données Lingua est un no-op (le job reste idempotent). Alternative rejetée : un job `purge_lingua` séparé — deux jobs à séquencer pour une même sémantique RGPD, et le précédent (plans, analytics) est l'extension du job unique.

### D8 — Socle consommé tel quel : flags par audience, analytics existant, Caddy inchangé
Feature flags : l'`EvalContext` dérive `app` de l'audience du jeton — un jeton `lingua` est scopé `lingua` automatiquement, aucune déclaration ; les futures bêtas Lingua se gèrent dans la console flags existante. Analytics : les clients émettront leurs évènements d'usage via `UsageService` avec les valeurs `platform` déjà au contrat du proto (`web`, `ios`, `macos`) — pas de nouveau pipeline. Observabilité : les services `lingua` passent sous l'`ObserveLayer` existante comme tout service tonic. **Caddy : vérifié contre le matcher** — les chemins gRPC `/cymbra.lingua.v1.<Service>/<Method>` sont disjoints de la liste `@http` (`/.well-known/*`, `/healthz`, `/web/*`, …) et tombent dans la branche par défaut → tonic h2c, qui porte aussi le gRPC-web + le preflight CORS. Le piège documenté (« préfixe absent du matcher → tonic répond 200 + grpc-status 12 ») ne mord que les routes **HTTP Axum** ; ce change n'en ajoute aucune, donc le Caddyfile ne bouge pas.

## Risks / Trade-offs

- [LWW + horloges client fausses : un appareil à l'heure décalée « gagne » des conflits] → tie-break déterministe par `device_id`, horodatage serveur de réception conservé pour l'audit, et clamp des horodatages futurs à la réception ; le pire cas reste corrigeable d'un clic (reposer le statut).
- [Origine `chrome-extension://<id>` : l'id change entre dev (unpacked) et store] → l'id publié est stable (clé au manifest) ; en dev, l'origine locale s'ajoute à `CYMBRA_ALLOWED_WEB_ORIGINS` de l'environnement de dev seulement. Firefox (`moz-extension://<uuid>` aléatoire par install) : non bloquant — les requêtes fetch d'une event page MV3 Firefox ne portent pas d'origine soumise à cette CORS de la même façon ; à vérifier à l'intégration cliente, le repli documenté étant un id d'origine par `browser_specific_settings`.
- [Poussée initiale volumineuse au premier sign-in (des années de statuts)] → lots bornés + reprise par offset d'outbox, portés par le protocole (idempotence par lot) ; l'ordre des ops préserve les horodatages, donc une interruption est reprise sans corruption.
- [Deux listes d'origines de plus en plus proches (`WEB_ORIGINS`, `ALLOWED_WEB_ORIGINS`)] → noms et commentaires d'env explicites (« cookie credentialed » vs « gRPC-web bearer ») ; refus délibéré de les fusionner tant que leurs sémantiques diffèrent.
- [Module lingua sur le même serveur : rayon d'explosion partagé] → même trade-off que music/plans, assumé par la trajectoire backend actée (pas de split avant le besoin) ; le crate garde la frontière (schéma + rôle + port) pour que le split reste un petit travail.
- [Backend livré avant tout client : risque de contrat inadapté] → le protocole est testé par deux clients **simulés** convergents (test d'intégration bout-en-bout), et les types viennent de `lingua-core` — le même vocabulaire que les stores clients réels.

## Migration Plan

1. Backend d'abord (déployable inerte) : crate + migrations + rôles + protos + services, `CYMBRA_LINGUA_DATABASE_URL` absente en prod → rien ne change pour personne.
2. Provision : `provision-lingua-role.sql` + extension du `search_path` admin + env prod (`ALLOWED_AUDIENCES`, `ALLOWED_WEB_ORIGINS`, client id Google) ; activer la variable DB.
3. Les clients arrivent au change `add-lingua-connected-clients` (extension puis app Apple).
4. Rollback : retirer `CYMBRA_LINGUA_DATABASE_URL` rend le module inerte ; aucun client ne dépend encore du serveur.

## Open Questions

- Le format exact du snapshot (un message unique vs stream par pages) — à trancher à l'implémentation selon les tailles réelles ; le contrat (ETag/version, réponse « inchangé ») ne bouge pas.
