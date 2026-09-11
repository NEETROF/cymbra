---
description: List this session's unknown English words and add them to your Lingua deck.
---

The user wants to review the new English vocabulary from the current session.

1. Run the Lingua binary against this session's transcript:

   `lingua vocab --transcript "$CLAUDE_PROJECT_DIR/../<the current transcript path>"`

   (The `Stop` hook already ingests each turn automatically; this command surfaces the
   session's unknown words with their dictionary form, gloss and rarity.)

2. Present the listed words to the user. For any they want to keep, add them to the deck
   with the `add_words` tool of the `lingua` MCP server (or
   `lingua vocab --transcript <path> --add word1,word2`). The source sentence is captured
   onto the card only at that point.

Never show the internal analysis jargon to the user — say « forme du dictionnaire » and
« mots différents ».
