-- The soundfont instrument column joins the score vocabulary (change:
-- add-drum-audio-channel).
--
-- `music.soundfonts.instrument` held free text defaulting to 'piano' — a fourth
-- spelling beside the scores' keyboard/percussion/unknown. Every consumer this
-- change creates (picker filters, the preview job's font choice, the family
-- verification) compares a score family against a font family, so the column is
-- migrated to the score vocabulary and constrained to the two families a stored
-- font can be. 'unknown' stays a score-only value: a font's family is verified
-- against its preset banks at the door, so "we could not tell" is not a state a
-- stored font can be in. The upload boundary keeps normalising legacy 'piano'
-- input to 'keyboard' permanently (see backend/server/src/soundfont.rs), so
-- shipped clients and old scripts keep working.
--
-- Reversible: no information is lost ('keyboard' rows were all 'piano').

UPDATE music.soundfonts SET instrument = 'keyboard' WHERE instrument = 'piano';

ALTER TABLE music.soundfonts ALTER COLUMN instrument SET DEFAULT 'keyboard';

ALTER TABLE music.soundfonts
    ADD CONSTRAINT soundfonts_instrument_family
        CHECK (instrument IN ('keyboard', 'percussion'));
