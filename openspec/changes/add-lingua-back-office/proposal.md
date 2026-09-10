# add-lingua-back-office — Cymbra Lingua : section admin du back-office (OPS)

## Why

Une fois le backend Lingua en place (change `add-lingua-id-sync` : schéma `lingua`, sync, `StatsService`), l'exploitation du produit est aveugle : aucun moyen de savoir combien de comptes utilisent Lingua, à quel rythme, sur quelles langues, ni quelles versions de packs de données circulent. Le back-office Vue est déjà la console d'administration de tous les produits (music, plans, flags, usage) — Lingua doit s'y brancher, pas inventer une console à part.

Le périmètre est **OPS uniquement, par décision produit** : des agrégats, le registre des packs, les flags. **Pas de vue support par compte** — les données Lingua (mots rencontrés, decks, historique de révision) décrivent ce qu'une personne lit ; c'est sensible par nature, et l'administrateur n'a aucun besoin opérationnel de les voir. Cette limite est une exigence de spec, pas une omission.

## What Changes

- **Nouvelle capability `admin-lingua-console`** : l'écran « Lingua » du back-office et les RPC admin qui le servent.
- **Nouveaux protos admin dans `backend/lingua/proto`** (fichier dédié `lingua_admin.proto`, service `LinguaAdminService`) : `AdminGetLinguaUsage` (tuiles agrégées), `AdminGetLinguaUsageSeries` (séries par jour), `AdminListDataPacks` (registre des packs). Derrière l'intercepteur d'auth **strict** existant, gatés `require_admin_in_scope(id, "lingua")` — le pattern des RPC admin de `backend/music` et de `backend/analytics`, avec le scope **lingua** explicite (leçon « séparation des pouvoirs » : un `music/admin` n'administre pas Lingua par accident).
- **Implémentation backend** dans `backend/lingua` : agrégats SQL sur le schéma `lingua` (pool `lingua_svc`), aucune nouvelle télémétrie — on agrège ce que la sync possède déjà. Aucune réponse ne contient d'identifiant de compte.
- **Registre des packs = manifeste committé** : `scripts/lingua-data` émet un `packs-manifest.json` (pack_version, analyzer_version, paire L2→L1, date de build CI, taille, NOTICE), embarqué par le backend au compile et servi tel quel. La distribution des packs reste **embarquée dans l'extension** en v1 : le registre est informatif et prépare l'OTA futur.
- **Écran BO « Lingua »** (`apps/back-office`, route `/lingua`) suivant strictement la skill `vue-frontend-architecture` : store Pinia derrière le seam `api()`/`setClientsForTest`, état async en unions `Async<T>` matchées exhaustivement, aucun appel API dans les composants. Pattern visuel de l'écran `/usage` (tuiles + séries temporelles + répartitions). Entrée de navigation et route gatées `adminScope: "lingua"` (précédent : `/takedowns`).
- **Flags Lingua : aucune UI nouvelle.** Les clés sont déclarées dans le registry backend (`KeyDef`, app `lingua`) et administrées par la console `/flags` **existante**.
- Vocabulaire UI : comme partout dans Lingua, le mot « lemme » n'apparaît jamais à l'écran (« mots appris », « mots différents », « forme du dictionnaire »).

## Capabilities

### New Capabilities
- `admin-lingua-console` : la section Lingua du back-office — autorisation scopée `lingua`, agrégats d'usage (tuiles + séries + répartition par langue étudiée), règle dure de vie privée (agrégats seuls, jamais de donnée par compte), registre des versions de packs en lecture, flags via la console existante, états async localisés.

### Modified Capabilities
_Aucune. La console `/flags` (`feature-flags-admin`) est consommée telle quelle — les clés Lingua ne sont que des déclarations de registry côté backend, déjà couvertes par `runtime-feature-flags`. L'authentification back-office (`back-office-admin-session`) et le modèle de rôles scopés sont consommés, pas modifiés (le scope `lingua` est une valeur de plus dans une liste existante, pas un nouveau comportement)._

## Impact

- **Produits** : **back-office** = tout le nouveau front (écran, store, nav, i18n en/fr) ; **Lingua backend** = nouveaux protos + RPC admin dans `backend/lingua` (dépend de `add-lingua-id-sync`, qui crée le crate, le schéma et la sync) ; **ID** = consommé (audience `back-office`, rôles scopés, `require_admin_in_scope`) ; **Music / Live / site** : intacts.
- **Dépendance** : `add-lingua-id-sync` doit être implémenté d'abord (schéma `lingua`, pool `lingua_svc`, `StatsService`). Si ce change n'a pas déjà déclaré le scope `lingua` (`SCOPES`/`APP_SCOPES` de `backend/platform` + attribution de rôles), le présent change le fait.
- **Arborescence** : `backend/lingua/proto/lingua_admin.proto` + `src` (agrégats, gating) ; `scripts/lingua-data` (émission du manifeste) ; `apps/back-office/src` (stores/lingua.ts, views/LinguaView.vue, transport, router, i18n) ; `apps/back-office/e2e/lingua.spec.ts`.
- **CI** : aucune nouvelle unité — `backend/lingua` est couvert par la lane `rust` (workspace) et `apps/back-office` par `back-office-check` ; `ci-units` inchangé. Le nouveau proto tombe sous le gate `buf breaking` du workflow `proto` (fichier nouveau : passe). Coverage ≥ 80 % des deux côtés ; les adaptateurs minces (`pg*.rs`, `grpc.rs`) suivent la convention d'exclusion existante, la logique d'agrégation reste host-testée.
- **Hors périmètre (explicitement)** : vue support par compte (rejetée pour la vie privée — pas « plus tard », rejetée), OTA des packs (le registre le prépare, ne le livre pas), écriture du registre depuis la CI vers la prod, nouvelle UI de flags, agrégats temps réel.
