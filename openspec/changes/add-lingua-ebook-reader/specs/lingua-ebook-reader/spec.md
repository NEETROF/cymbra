# lingua-ebook-reader — bibliothèque et lecture d'ebooks

## ADDED Requirements

### Requirement: Import local d'epub et de texte brut
L'extension SHALL importer des fichiers `.epub` et `.txt` par glisser-déposer ou sélecteur de fichier sur la page bibliothèque, et ne SHALL accepter aucun autre format (pas de PDF). Le fichier importé ne SHALL jamais quitter l'appareil — aucune requête réseau pendant l'import, l'analyse ou la lecture. Les chapitres d'un epub SHALL suivre l'ordre du spine de l'OPF ; un `.txt` SHALL être traité comme un chapitre unique.

#### Scenario: Import d'un epub par glisser-déposer
- **WHEN** l'utilisateur dépose un epub du domaine public sur la page bibliothèque
- **THEN** le livre apparaît dans la bibliothèque avec titre et auteur (métadonnées OPF), son profil est calculé localement et aucune requête réseau n'est émise

#### Scenario: Format non supporté refusé
- **WHEN** l'utilisateur tente d'importer un PDF
- **THEN** l'import est refusé avec un message expliquant que seuls les fichiers epub et texte brut sont supportés

#### Scenario: Epub mal formé sans casse
- **WHEN** l'utilisateur importe un epub corrompu
- **THEN** l'import de ce livre échoue avec un message clair et la bibliothèque reste fonctionnelle

### Requirement: Refus des epubs verrouillés par DRM
L'extension SHALL détecter qu'un epub est chiffré par une mesure de protection (DRM) et SHALL refuser son import avec un message clair et sans jargon expliquant que les livres verrouillés ne peuvent pas être analysés ; elle ne SHALL en aucun cas contourner ni retirer une mesure de protection, et rien de ce fichier ne SHALL être stocké.

#### Scenario: Epub sous DRM refusé
- **WHEN** l'utilisateur importe un epub dont le contenu est chiffré
- **THEN** l'import échoue avec un message expliquant que ce livre est verrouillé par son vendeur, sans tentative de déchiffrement, et le fichier n'est pas conservé

### Requirement: Bibliothèque à bandes de couverture avec provenance
La bibliothèque SHALL lister les livres importés avec, pour chacun : titre, auteur, une bande de couverture approximative (« ~87 % ») — jamais de décimales ni de verdict « prêt/pas prêt » — la provenance du calcul (« ton exemplaire ») et la position de lecture. Les bandes SHALL refléter les changements de statuts de mots sans ré-import. Un livre sans contenu suffisant dans la langue étudiée SHALL être signalé non analysable, sans pourcentage affiché.

#### Scenario: Bande arrondie, jamais de décimales
- **WHEN** la couverture calculée d'un livre est 87,4 %
- **THEN** la bibliothèque affiche « ~87 % » avec la mention « ton exemplaire », jamais « 87,4 % »

#### Scenario: Progression reflétée sans ré-import
- **WHEN** l'utilisateur marque connus plusieurs mots depuis une page web puis revient à la bibliothèque
- **THEN** les bandes reflètent le nouvel état de connaissance sans nouvelle analyse du fichier

### Requirement: Fiche livre avec courbe par chapitre et chemins vers les seuils
La fiche d'un livre SHALL afficher : la bande de couverture globale, la couverture par chapitre, les repères « ≈N mots pour 95 %, M pour 98 % » (seuils de lecture assistée et de lecture confortable), et une rampe par chapitre indiquant combien de mots débloquent les premiers chapitres (« X mots débloquent les chapitres 1-3 »).

#### Scenario: Repères de seuils sur l'exemplaire
- **WHEN** l'utilisateur ouvre la fiche d'un livre couvert à ~91 %
- **THEN** la fiche affiche le nombre approximatif de mots à apprendre pour atteindre 95 % puis 98 %, calculés sur les tokens de son exemplaire, et la rampe par chapitre

### Requirement: Gap deck généré depuis la fiche livre
La fiche SHALL permettre de générer un deck des mots inconnus du livre : trié par fréquence décroissante dans le livre, hapax exclus, taille plafonnée, chaque carte portant la forme du dictionnaire, une phrase de contexte extraite de l'exemplaire local au moment de la génération, et la provenance (titre du livre, chapitre). Les cartes SHALL rejoindre les decks et la révision FSRS existants sans schéma parallèle, et l'interface ne SHALL pas employer le mot « lemme » (« mots différents », « forme du dictionnaire »).

#### Scenario: Génération plafonnée, triée, sans hapax
- **WHEN** l'utilisateur génère le gap deck d'un livre comptant 1 200 mots inconnus dont 400 n'apparaissent qu'une fois
- **THEN** le deck contient au plus le plafond de cartes, les mots les plus fréquents du livre d'abord, aucun mot n'apparaissant qu'une fois, et chaque carte porte sa phrase du livre et sa provenance

#### Scenario: Cartes révisables comme les autres
- **WHEN** un gap deck vient d'être généré
- **THEN** ses cartes apparaissent dans le compteur de dues, se révisent dans le side panel et figurent dans l'export Anki comme toute autre carte

### Requirement: Lecture instrumentée par le pipeline de surlignage existant
Le reader SHALL afficher le livre chapitre par chapitre et SHALL appliquer au DOM du chapitre le même pipeline que les pages web : surlignage des mots inconnus et « en cours » via la CSS Custom Highlight API, popup de mot au clic et capture de sélection au raccourci clavier — sans logique d'analyse dupliquée. Toute action de statut SHALL re-peindre le chapitre et se propager aux autres surfaces (onglets, side panel).

#### Scenario: Chapitre surligné comme une page web
- **WHEN** l'utilisateur ouvre un chapitre dans le reader
- **THEN** les mots inconnus y sont surlignés comme sur une page web, et cliquer un mot ouvre le même popup (forme du dictionnaire, glose, rareté, actions de statut)

#### Scenario: Capture d'expression dans le reader
- **WHEN** l'utilisateur sélectionne une expression dans un chapitre et presse le raccourci de capture
- **THEN** une carte d'expression est créée avec la phrase du livre et la provenance (titre, chapitre)

### Requirement: Rendu assaini du contenu epub
Le reader SHALL rendre le XHTML des chapitres après un assainissement structurel à liste blanche : scripts, styles embarqués, gestionnaires d'événements et références à des ressources externes retirés — le contenu d'un epub est du contenu tiers non fiable rendu dans l'origine de l'extension. Aucune ressource référencée par l'epub ne SHALL déclencher de requête réseau.

#### Scenario: Epub hostile neutralisé
- **WHEN** un epub contenant un script, un gestionnaire `onclick` et une image pointant vers un serveur externe est ouvert dans le reader
- **THEN** le texte du chapitre s'affiche, aucun script ne s'exécute et aucune requête réseau n'est émise

### Requirement: Position de lecture et réglages de confort persistés
Le reader SHALL persister par livre la position de lecture (chapitre et position dans le chapitre) et la restaurer à l'ouverture, et SHALL offrir des réglages de confort minimaux — taille du texte, thème clair/sombre — appliqués via les tokens Cymbra et persistés.

#### Scenario: Reprise de lecture
- **WHEN** l'utilisateur rouvre un livre quitté au chapitre 5
- **THEN** le reader s'ouvre au chapitre 5, à la position quittée, avec ses réglages de confort

### Requirement: Stockage local versionné, purgé à la suppression
Les fichiers importés et leurs profils SHALL être stockés dans l'IndexedDB de l'origine de l'extension (jamais `chrome.storage.local` pour les blobs), sous un schéma versionné avec migration ascendante. Supprimer un livre SHALL purger son fichier, son profil et sa position de lecture ; les cartes déjà créées SHALL rester dans les decks.

#### Scenario: Suppression purgante
- **WHEN** l'utilisateur supprime un livre de la bibliothèque
- **THEN** le fichier, le profil et la position disparaissent du stockage local, et les cartes issues de son gap deck restent révisables
