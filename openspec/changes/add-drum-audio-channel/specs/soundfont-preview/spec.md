## MODIFIED Requirements

### Requirement: Server-rendered preview clip

The server SHALL be able to render a short **preview clip** for a SoundFont by
synthesizing a **fixed sample sequence chosen by the font's instrument family**
with that font, headlessly (no audio device), to a PCM buffer, then encoding it
to a compact, universally playable audio container: a `keyboard`-family font
plays the existing fixed melodic phrase on the melodic channel, and a
`percussion`-family font plays a fixed short **groove** on the **drum channel**
(`music-drum-audio`) — the melodic phrase through a kit font would be silence or
nonsense, since a kit's presets live in bank 128. Within a family the sequence
is the same for every font, so clips stay comparable. The synthesis SHALL run on
the server; the raw font bytes SHALL NOT leave the server to produce a preview.

The sample sequences, PCM shaping, and encoding SHALL be host-testable pure
helpers (the device-free synthesizer call and object I/O may be coverage-excluded
glue), so rendering the same font with its family's sequence is deterministic.

#### Scenario: Rendering produces a deterministic clip

- **WHEN** a font's bytes are rendered with its family's fixed sample sequence
- **THEN** a non-empty audio clip is produced
- **AND** rendering the same font with the same sequence again yields an equivalent clip (same duration/format)

#### Scenario: A percussion font's preview is a groove

- **WHEN** a `percussion`-family font's preview is rendered
- **THEN** the fixed groove is synthesized on the drum channel and the clip is
  audibly a drum pattern, not silence

#### Scenario: Keyboard fonts keep their phrase

- **WHEN** a `keyboard`-family font's preview is rendered
- **THEN** the existing melodic phrase is synthesized on the melodic channel,
  unchanged by this change
