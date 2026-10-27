# Cymbra Lingua — the Italian glossary

One Italian term per product concept, across every surface that speaks Italian: the extension's
interface (`apps/lingua-extension/src/i18n/it/`), its word card, `_locales/it`, the Safari app's
activation page (`copy.js`) and sign-in sheet (`SignInCopy`), the site's `/it/` pages and the store
listings. Written by change 44 of the [Italian programme](italian-programme.md)
(`review-lingua-italian-wording`, I2), after the reviewer agent's pass over all of them
([italian-wording-review.md](italian-wording-review.md)).

The data is [`apps/lingua-extension/test/italian-glossary.json`](../../apps/lingua-extension/test/italian-glossary.json);
this page presents it, concept by concept, and `test/italian-glossary.spec.ts` fails if a concept is
missing here. It is the reference for every later Italian string: changes 31, 50, 54, 55 and 56, and
any change that adds Italian text.

## How a term was chosen (D1)

Where the surfaces disagreed, in this order:

1. the words change 39 pinned and its test holds equal between `_locales/it` and Réglages:
   « riquadro laterale », « pannello di ripasso », « acquisisci la selezione »;
2. Cymbra Music's `app_it.arb` for Cymbra-wide terms (« Accedi », « Esci », « Elimina account »);
3. the term the interface uses;
4. a platform's own word, where better known.

Two departures are recorded with their reason: « nome utente », where Music says « Handle »
(`username`), and the site's « Termini di utilizzo », where Music's footer says « Termini di
servizio » (`terms-of-use`).

## What the test reads (D3)

`test/italian-glossary.spec.ts` reads every Italian message of the catalogue's text surfaces
(through `test/support/catalogue.ts`'s `texts()`, `grammar.ts` left out: it renders tags), of
`_locales/it` (each `message`, never its `description`) and of `copy.js` (its fragments' markup
stripped), each beside its French under the same key. Texts are read in NFC, the typographic
apostrophe as `'`, no-break spaces and runs of whitespace as one space, the brand « Cymbra Lingua »
masked; a form matches whole and case-free, bounded by letters
(`(?<![\p{L}\p{M}\p{N}])…(?![\p{L}\p{M}\p{N}])`, flags `iu`).

1. Where the French names a concept, the Italian names it with an accepted form.
2. No Italian text uses a refused variant, whether or not its French names the concept.
3. Every concept is named by some French text; an exception that no longer applies fails.

A refused form is one that only ever means the concept (« pannello laterale », « traduzione
avanzata »), never a word with other senses (« grado », « conto », « opzioni » were left out for
that). Should a later string need a refused form in another sense, it takes an exception in the JSON,
with its reason; the test fails the exception once it no longer applies.

The site, the listings and `SignInCopy` are read by the reviewer agent's pass, not by the test; their
own terms are the last table below.

### Written for their audience

These keys are left out of rule 1 by design (not as exceptions); rule 2 still reads them.

| Key | Why |
|---|---|
| `_locales.extensionDescription` | Written for Italian speakers studying English (change 39, change 46's D1), not translated from the French. |
| `copy.lede` | Names the languages an Italian reader studies (English, en-it), not the French page's three (change 40's D3). |
| `copy.step3` | Stops at choosing the level for English alone: an Italian reader has one pair, so the French sentence on ticking another language in Réglages › Langue is not there (change 40's D3). |

## The concepts the test holds

| Id | Concept | French | Italian | Refused | Why |
|---|---|---|---|---|---|
| `review-panel` | the review panel on the page (the drawer) | « panneau de révision », « tiroir » | « pannello di ripasso », « pannello » | « pannello di revisione », « cassetto » | Change 39's word (precedence 1): `_locales/it`'s commandToggleDrawer and Réglages' shortcut line say « pannello di ripasso sulla pagina », held equal by locales-shortcuts.spec.ts; « pannello » alone where the French says « tiroir ». |
| `side-panel` | the browser's side panel | « panneau latéral » | « riquadro laterale » | « pannello laterale » | Change 39's word (precedence 1): Chrome's Italian name for its side panel (26 strings against 2), so the shortcuts page and Réglages agree; « pannello » stays for Lingua's own panels. Firefox's « barra laterale » is Firefox's own word, not refused. |
| `capture` | capturing the selection | « capturer », « capture », « capturé », « capturée » | « acquisisci », « acquisire », « acquisita », « acquisito » | « cattura », « catturare », « catturata », « catturato » | Change 39's word (precedence 1): Réglages' verb, « Acquisisci la selezione »; its draft's « Cattura » suggests a screenshot in Italian interfaces. |
| `deck` | the deck of cards | « deck » | « mazzo », « mazzi » | « deck » | The interface's word (I6 rule 3, change 33): « mazzo », as a deck of cards; the French keeps the English « deck ». |
| `card` | a card of the deck or the word card | « carte », « cartes » | « carta », « carte » | « flashcard », « flashcards » | The interface's word (I6 rule 3): « carta », never « scheda », which is the browser's tab. « flashcard » stays a search keyword of the App Store listing, not the interface's name. « Carta » is also the Paper theme's name (French « Papier »), capitalised where it names the theme (« Tema Carta: sfondo »), so no rule refuses « carta ». |
| `tab` | the browser's tab | « onglet », « onglets » | « scheda », « schede » | « tab », « linguetta » | Chrome's and Firefox's Italian word (a platform's word, precedence 4); never used for a card. Safari's Italian may call its tabs « pannelli » (unverified, the owner's checklist E12): the activation page keeps « schede » until the dogfood says otherwise, « pannello » being Lingua's panels. A settings tab is named by its path, « Impostazioni › Dati », not « scheda ». |
| `review` | reviewing, a review | « réviser », « révise », « révision », « révisions », « revoir » | « ripassa », « ripassale », « ripassare », « ripasso », « ripassi » | « revisione », « revisioni », « revisionare » | The interface's word (I6 rule 3): « ripasso / ripassare », a learner's review; « revisione » is an inspection or an edit. « Da rivedere » names a missed card shown again, a second form kept on purpose (change 44's pass). Exceptions: `review.repeat()` — « À revoir · 2ᵉ passage » is a missed card shown again: « Da rivedere · 2º passaggio », kept apart from the due count's « da ripassare » on purpose (change 44's pass). |
| `settings` | the extension's settings (Réglages), and the device's Settings app | « réglages », « réglage » | « impostazioni », « impostazione » | — | Music's word (precedence 2), « Impostazioni », which is also iOS's and Chrome's. Safari's « preferenze » before macOS 13 answers the French « préférences », not this concept. A settings tab is named by its path, as the French writes it (« Impostazioni › Dati », « Impostazioni → Dati »), never « scheda Dati »; the activation page's step 3 will name « Impostazioni › Lingua » so when changes 50 and 54 give it a second sentence. |
| `statistics` | the statistics page and tab | « statistiques », « stats » | « statistiche », « stats » | — | « Statistiche » for the page and the side panel's tab; « Stats » where the French itself writes the clipping « Stats » (the HUD's and the drawer's narrow buttons), as the Spanish does — the owner's provisional choice of 2026-10-26, kept by change 44 (its report). |
| `sign-in` | signing in | « se connecter », « connecte-toi », « reconnecte-toi », « te connecter », « connexion » | « accedi », « accedere », « accesso » | « login », « log in », « loggati » | Music's word (precedence 2): « Accedi »; « Accesso effettuato » for « Connecté », which never assumes the reader's gender (I6 rule 2). « sessione » names both the sign-in session and a review session (French « session » / « séance »); the two never share a sentence. Exceptions: `account-errors.unavailable` — « Vérifie ta connexion » is the network connection: « Controlla la connessione ».; `translation.failedNetwork` — « pas de connexion » is the network connection: « nessuna connessione ». |
| `sign-out` | signing out | « se déconnecter », « déconnecté » | « esci », « disconnessione » | « logout », « log out », « disconnettiti » | Music's word (precedence 2): « Esci »; « Disconnessione effettuata » for « Tu es déconnecté », free of the reader's gender. |
| `account` | a Cymbra account | « compte Cymbra », « ton compte », « un compte », « ce compte », « le compte », « mon compte », « autre compte », « comptes » | « account » | — | Music's word (precedence 2): « account », invariable. The French « compte » is also the verb « compter », so only its nominal uses are forms here. |
| `delete` | deleting (an account, a book) | « supprimer », « supprime », « supprimé », « suppression » | « elimina », « eliminare », « eliminato », « eliminata », « eliminazione » | — | Music's « Elimina account » (precedence 2); « Cancella » stays for « Effacer » (erasing data), a different act. |
| `linked-accounts` | the sign-in methods linked to an account | « comptes connectés », « méthode de connexion », « méthodes de connexion » | « account collegati », « metodo di accesso », « metodi di accesso » | « account connessi » | The interface's words (change 33), « Account collegati », with sign-in's « accesso ». |
| `password` | a password | « mot de passe » | « password » | « parola d'ordine », « parola segreta » | Music's word (precedence 2): « password », feminine (« la password »). |
| `email` | an email address or message | « email », « e-mail », « emails », « e-mails » | « email » | « e-mail », « mail », « posta elettronica » | Music's sign-in word (« Continua con l'email », precedence 2) and the interface's: « email », feminine. Music elsewhere writes « e-mail », its own inconsistency, out of this change's scope. |
| `username` | the username (French « pseudo ») | « pseudo » | « nome utente » | « pseudonimo », « nickname », « handle » | The interface's word (change 33), also the site's privacy policy (change 41's review: « pseudonimo » read as an assumed name). An exception to precedence 2, kept by change 44: Music's arb says « Handle », not an Italian word for it, while Italian interfaces that show an @-name say « nome utente »; the value always shows with its « @ », so a reader knows it in both products. Aligning Music is a Music change. |
| `known` | a word marked known | « connu », « connue », « connus », « connues » | « conosciuta », « conosciute », « conosciuto », « conosciuti » | — | The interface's word: « parola conosciuta », agreeing with « parola », never with the reader. |
| `unknown` | an unknown word | « inconnu », « inconnue », « inconnus », « inconnues » | « sconosciuta », « sconosciute », « sconosciuto », « sconosciuti » | — | The interface's word: « parole sconosciute »; « ignota » is literary, but not refused: it has other senses (« causa ignota »). |
| `learning` | a word being learnt | « en cours d'apprentissage », « mots en cours », « (en cours) » | « in apprendimento » | « in corso di apprendimento » | The interface's words: « in apprendimento », a status, where « in corso » would read as « under way ». |
| `highlight` | highlighting a word | « surligner », « surligne », « surligné », « surlignée », « surlignés », « surlignées », « surlignage » | « evidenzia », « evidenziare », « evidenziata », « evidenziate », « evidenziato », « evidenziati », « evidenziazione » | — | The interface's word (I6 rule 3): « evidenziare / evidenziazione »; the underline style is « sottolineatura », a setting of its own. |
| `level` | a reader's level | « niveau », « niveaux » | « livello », « livelli » | — | The interface's word (I6 rule 3). |
| `level-scale` | the level scale's acronym | « CEFR », « CECRL », « CECR » | « QCER » | « CEFR », « QCRE », « CECR » | I6: « QCER », the Quadro comune europeo di riferimento's Italian acronym, as M19 gave each language its own. |
| `pack` | a language pack (the dictionary data) | « pack », « packs » | « pacchetto », « pacchetti » | « pack » | The interface's word (I6 rule 3). |
| `extended-translation` | the on-device translation setting | « traduction étendue » | « traduzione estesa » | « traduzione avanzata », « traduzione ampliata » | The interface's words (I6 rule 3). |
| `online-translation` | the opt-in online translation | « traduction en ligne », « traduction automatique en ligne » | « traduzione online », « traduzione automatica online » | « traduzione in linea » | The interface's words (I6 rule 3), « online » as Italian writes it. |
| `read-aloud` | reading aloud | « voix haute » | « ad alta voce » | « a voce alta » | The interface's words: « Lettura ad alta voce ». Exceptions: `languages.french.preview` — The French voices' preview is French in every interface (add-lingua-french-read-aloud D4). |
| `offline` | offline | « hors ligne » | « offline » | « fuori linea », « non in linea » | The interface's word, as `_locales/it` writes it (« Offline e senza tracciamento »). |
| `sync` | synchronising | « synchroniser », « synchronise », « synchronisent », « synchronisé », « synchronisation », « sync » | « sincronizza », « sincronizzare », « sincronizzano », « sincronizzato », « sincronizzati », « sincronizzazione » | « sync » | The interface's word; the French clipping « sync » is written out. |
| `backup` | a backup | « sauvegarde », « sauvegarder » | « backup » | « copia di sicurezza » | The interface's word, as Italian software writes it (« Esegui il backup »). |
| `restore` | restoring a backup | « restaurer », « restaurée », « restauré » | « ripristina », « ripristinare », « ripristinato », « ripristino » | — | The interface's word; « Recupero dal server » keeps the server restart apart (change 33's review). |
| `library` | the book library | « bibliothèque » | « biblioteca » | « libreria » | The interface's word: « biblioteca », the reader's collection of books. « libreria » can mean one too (Treccani's first sense, « raccolta di libri »; Apple Libri's word, unverified), but it also reads as a bookshop or a bookcase, so the interface keeps one word and refuses the other. |
| `badge` | the percentage pill on the page | « pastille » | « badge » | « pillola », « bollino » | The interface's word (change 33), as the site's captions and the App Store text write it. The toolbar icon's count has no noun in Italian (« le carte da ripassare sull'icona »): a string that names it must not call it « badge » too (change 44's pass kept « badge » for the pill rather than « indicatore »). |
| `true-percentage` | the page's percentage, counted from what the reader marked | « pourcentage honnête » | « percentuale reale » | « percentuale onesta » | `_locales/it`'s word (change 39): with a quantity « onesta » reads "fair, reasonable" (Treccani, « onesto »); the site's « Una percentuale onesta » was corrected by change 44. |
| `native-language` | the reader's native language | « langue maternelle » | « lingua madre » | « lingua materna », « lingua nativa » | The interface's words: « lingua madre », the current usage. |
| `studied-language` | a studied language | « langue étudiée », « langues étudiées » | « lingua studiata », « lingue studiate » | — | The interface's words. |
| `expression` | a multi-word expression | « expression », « expressions » | « espressione », « espressioni » | « locuzione », « locuzioni » | The interface's word, as the card's title names it. |
| `colour-set` | a colour preset | « jeu de couleurs », « jeux de couleurs », « préréglage », « préréglages » | « set di colori » | « preset » | The interface's words (change 33's review unified « preset » into « set di colori »). |
| `steps` | the usage counts' steps | « étape », « étapes » | « tappa », « tappe » | — | The interface's word (change 33's review unified « passaggi » into « tappe »); « passaggio » stays free for its other senses (a card's « 2º passaggio »), so it is not refused. |
| `usage-counts` | the anonymous usage counts | « comptage », « comptages » | « conteggio », « conteggi » | — | The interface's word, as the listings write it (« conteggi anonimi »). |
| `to-sort` | sorting a book's looked-up words | « trier », « trié », « triés », « à trier » | « smistare », « smistato », « smistate », « smistati » | — | The interface's word. |
| `practice` | the review app's practice round | « entraînement » | « esercizio » | « allenamento » | The interface's word. |

## Read by the pass, not the test

The site's and Music's terms that the extension's surfaces never meet: the reviewer agent reads them
against this table.

| Id | Concept | French | Italian | Refused | Why |
|---|---|---|---|---|---|
| `english-tense-names` | English tenses named on the card for an Italian reader | « prétérit », « présent simple » | « passato semplice », « presente semplice », « forma in -ing », « participio passato » | « simple past », « past simple », « present simple » | Change 34's D3 (`src/i18n/it/grammar.ts`, read by the pass and change 34's snapshot, not this test): the Italian Wiktionary's names, read whole inside an Italian sentence (« del presente semplice »). Italian schoolbooks of English often keep the English names; change 44's pass recommends keeping D3, the listings no longer claim it is « come si impara a scuola », and change 55's Italian readers settle it. |
| `feedback` | Music's immediate feedback | « retour immédiat », « retour » | « riscontro » | — | The site's word (`/it/`, `/it/music/`), the Accademia della Crusca's Italian for « feedback »; Music's Italian store copy says « feedback », Music's string, out of this change's scope. |
| `score` | Music's sheet music | « partition » | « spartito » | — | The site's word throughout; Music's arb mixes « spartito » and « partitura », Music's to align. |
| `terms-of-use` | the terms of use | « Conditions d'utilisation » | « Termini di utilizzo », « Termini » | « Termini di servizio » | The site's page title and the French source (« Conditions d'utilisation »; the Spanish « Condiciones de uso »); the Cymbra ID e-mails' footer shortens it to « Termini ». Music's footer « Termini di servizio » follows its English arb ("Terms of Service") — Music's string, out of this change's scope. |
| `wait-mode` | Music's Wait Mode | « Mode Attente » | « modalità attesa » | « Wait Mode » | Music's own label (`leaderboardModeWait`, precedence 2), written lower case mid-sentence on the site (change 41's review). Music's store copy « Modalità Attesa » and two arb strings saying « Wait Mode » are Music's strings, out of this change's scope. |

## Typography the glossary assumes

- « tu », never « Lei »; no participle that assumes the reader's gender (I6).
- Guillemets « » with no inner space; no space before « : ; ? ! »; the French « — » between two
  clauses becomes a colon or a full stop.
- One apostrophe in the extension's Italian, U+0027 (change 34's D4, held by `test/i18n.spec.ts`):
  it takes precedence over the reviewer's « as the French writes it at that place ». The activation
  page keeps U+2019 where its French does (change 40).
- « 5000 » under five digits, « 20.000 », « 96% », « QCER ».
