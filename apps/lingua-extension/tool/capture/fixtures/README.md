# Capture fixtures

What `yarn capture` (`tool/capture/`, change `add-site-lingua-showcase`) reads with the extension.
Nothing here is shipped in a package.

## `pages/` — articles written for the captures

`en.html`, `es.html` and `fr.html` are one article, « The quiet return of the night train », written
for the purpose in each studied language (about 450 words, B1–B2 vocabulary), on a neutral layout
(`page.css`) with no publication's name, logo or brand. `pointer.js` draws a pointer for the demo
video only (`?pointer`): a recorded video shows no mouse. They are the repository's own work,
licensed like it (Apache-2.0).

## `books/` — public-domain chapters

Each file holds a few `Name: value` lines, a blank line, then the chapter's paragraphs; the tool
assembles it into an EPUB 3 (`tool/capture/epub.mjs`). Each text is the original-language text, not a
translation, and is in the public domain in France, Spain, the European Union and the United States:
it was published in the nineteenth century, and its author died more than seventy years ago — more
than eighty, the term Spain keeps for authors who died before 1987:

| File     | Work                                                                          | Source printing                                                                                       |
| -------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `en.txt` | Jane Austen (1775–1817), _Pride and Prejudice_, chapter I                     | London: George Allen, 1894 (illustrated by Hugh Thomson)                                              |
| `es.txt` | Benito Pérez Galdós (1843–1920), _Marianela_, chapter I, « Perdido »          | Madrid: Imprenta y Litografía de La Guirnalda, 1878                                                   |
| `fr.txt` | Jules Verne (1828–1905), _Le Tour du monde en quatre-vingts jours_, chapter I | Paris: J. Hetzel et Cie, Bibliothèque d'éducation et de récréation, printed by Gauthier-Villars, 1873 |

The texts were taken from volunteer transcriptions of those printings and committed without any
distributor's header, licence or name. Only the
plain-text conventions were undone: illustrations' captions dropped, `_italics_` marks removed, and
`--` written as the dash it stands for. No modern annotated or critical edition was used.

## `state/` — the reader's state, recorded by the product

`fr.json`, `en.json` and `es.json` are backups in the extension's own format, written by Réglages ›
Données › Sauvegarder during `yarn capture --record-seed` (`tool/capture/plan.mjs` says what it
does): the native language (which sets the interface language), two studied languages at level B1,
about twelve cards with the sentences they were met in, a few dozen words marked known, a short
review, and the book's names marked « Ignorer » in the EPUB reader. They are re-recorded when the
fixtures or the plan change; an older one is migrated forward on restore, as a reader's backup is.
The tool only pretty-prints the file the product saves, and Prettier leaves it so. Before restoring
one, the tool moves its dates by the whole days between its recording and the run
(`tool/capture/seed.mjs`), so that every run finds the same cards due.
