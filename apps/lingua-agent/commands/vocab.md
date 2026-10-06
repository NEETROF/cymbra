---
description: List this session's unknown words and add them to your Lingua deck.
---

The user wants to review the new vocabulary from the current session, in each language they
study.

1. Run the Lingua binary against this session's transcript:

   `lingua vocab --transcript "$CLAUDE_PROJECT_DIR/../<the current transcript path>"`

   (The `Stop` hook already ingests each turn automatically; this command surfaces the
   session's unknown words with their dictionary form, gloss and rarity.)

2. Present the listed words to the user, under their language when there are several. For
   any they want to keep, add them to the deck of their language with the `add_words` tool
   of the `lingua` MCP server, passing `language` (`en`, `es`) when several languages are
   followed (or `lingua vocab --transcript <path> --add word1,word2`, with
   `--language <tag>` for a word listed in two languages). The source sentence is captured
   onto the card only at that point.

Never show the internal analysis jargon to the user — say « forme du dictionnaire » and
« mots différents ».
