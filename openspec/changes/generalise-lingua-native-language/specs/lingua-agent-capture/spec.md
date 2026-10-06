## MODIFIED Requirements

### Requirement: The languages followed are the packs installed
The plugin SHALL follow every studied language whose pack is installed in its data directory and glossed in the plugin's native language, the English pack staying at `pack.lingua`, with no other setting. The plugin's native language SHALL be that of `pack.lingua` when the core reads it, and otherwise that of the first pack it reads in file-name order. A pack that cannot be loaded, or that is glossed in another native language, SHALL be skipped without disrupting the agent. A plugin with only one pack installed SHALL behave as it did before it followed several languages.

#### Scenario: Installing the Spanish pack
- **WHEN** the user copies `es-fr.lingua` beside `pack.lingua`
- **THEN** the plugin follows English and Spanish from the next reply, with no setting changed

#### Scenario: A pack that does not load
- **WHEN** a pack in the data directory is damaged or built for another analyser generation
- **THEN** it is skipped and the other packs are followed

#### Scenario: A pack of another native language
- **WHEN** the user copies `es-en.lingua` beside `pack.lingua` and `es-fr.lingua`
- **THEN** it is skipped, the plugin follows English and Spanish with French glosses as before, and `/vocab` names the file it skipped

#### Scenario: English alone
- **WHEN** only `pack.lingua` is installed
- **THEN** the statusline, `/vocab` and the MCP tools answer as before, and no tool asks for a language
