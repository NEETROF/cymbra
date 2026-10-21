# Spec Delta

## ADDED Requirements

### Requirement: Italian's acronyms are a reviewed table, glossed through their entry or their expansions
Italian's reference reduction (it-fr's) and every pair glossing Italian SHALL read an Italian word as an acronym only from one reviewed table of an Italian rule module they both import, each row naming the acronym as written — wholly in capitals, two letters or more —, the expansions or the sense the English Wiktionary's Italian section's own entry for it gives, its part of speech and its reason; a row SHALL exist only where the acronym's lowercase is a form of the tables. The reference's tables SHALL hold each row as a form in capitals that is its own lemma, with no rank and no level, and as a dictionary word; the builder SHALL accept a form holding a capital only in a pack studying French or Italian, wholly in capitals, two letters or more and its own lemma, and SHALL refuse any other form holding a capital. A pair SHALL gloss a row from its own gloss of the acronym's entry when its edition glosses it, else from its own glosses of the row's expansions, each cut to its first sense and joined, an expansion it does not gloss passed over; a row it glosses nowhere SHALL have no gloss.

#### Scenario: The rows
- **WHEN** the tables are reduced with the reviewed table
- **THEN** each row — `NATO`, `USA`, `ANSA`, as the owner settles them — is a form in capitals of `tables/it/forms.tsv` mapping to itself, a dictionary word of `lexical.tsv`, with no rank and no level, and its lowercase stays a form of its own word

#### Scenario: A pair glosses the entry
- **WHEN** it-fr glosses `NATO` and the French Wiktionary's Italian section glosses the entry
- **THEN** the row's gloss is that gloss, cut to its first sense

#### Scenario: A pair glosses the expansions
- **WHEN** a row's entry has no gloss in the pair's edition but its expansions are glossed
- **THEN** the row's gloss is the expansions' first senses, in the row's order, joined

#### Scenario: A stray capital
- **WHEN** a form holding a capital that is not wholly in capitals, or not its own lemma, reaches the builder
- **THEN** the build fails naming the form

### Requirement: Italian's dictionary words leave its names out
Italian's dictionary words SHALL be its reference pair's glossed lemmas less those the reference's edition — the French Wiktionary's Italian section — files as a proper noun in every sense run, by the one rule the three other studied languages follow; a word with a common sense beside a name's SHALL stay a word. The checks over the committed tables SHALL hold `tables/it/lexical.tsv` to it, and the Italian names rule SHALL set a document's names aside as a dictionary word never is.

#### Scenario: A name
- **WHEN** it-fr glosses `roma` by a proper noun's senses alone
- **THEN** `roma` is not a dictionary word, carries its gloss, and a selection still opens its card

#### Scenario: A word beside a name
- **WHEN** it-fr glosses `marco` by a name's sense and a noun's (the coin)
- **THEN** `marco` is a dictionary word

#### Scenario: The checks
- **WHEN** `tables/it/lexical.tsv` lists a lemma it-fr glosses by proper-noun senses alone, or leaves out one it glosses otherwise
- **THEN** the checks fail, naming it-fr and the lemma
