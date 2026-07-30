// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Key-signature pitch inference.
//!
//! A minimal MusicXML file may draw an armure (`<fifths>`) yet never mark its
//! notes with `<alter>`. Only for such a file — one with no alteration data at
//! all — does the parser apply the key signature to every note (see
//! [`crate::parse`]). Conforming exporters always emit `<alter>`, so they are
//! never touched; re-deriving their bare notes from the key signature would
//! mis-sound legitimately natural notes. Pure and host-testable.

/// Diatonic order in which sharps are added to a key signature (F♯, C♯, G♯, …).
const SHARP_ORDER: [char; 7] = ['F', 'C', 'G', 'D', 'A', 'E', 'B'];
/// Diatonic order in which flats are added to a key signature (B♭, E♭, A♭, …).
const FLAT_ORDER: [char; 7] = ['B', 'E', 'A', 'D', 'G', 'C', 'F'];

/// Alteration (semitones) the key signature imposes on `step`.
///
/// `fifths > 0` sharpens the first `fifths` steps of the sharp order (+1);
/// `fifths < 0` flattens the first `-fifths` steps of the flat order (-1); the
/// count is clamped to 7. Steps outside the affected set stay natural (0).
pub fn key_signature_alter(fifths: i32, step: char) -> i32 {
    let step = step.to_ascii_uppercase();
    if fifths > 0 {
        let n = fifths.min(7) as usize;
        i32::from(SHARP_ORDER[..n].contains(&step))
    } else if fifths < 0 {
        let n = (-fifths).min(7) as usize;
        -i32::from(FLAT_ORDER[..n].contains(&step))
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_key_signature_is_all_natural() {
        for step in ['A', 'B', 'C', 'D', 'E', 'F', 'G'] {
            assert_eq!(key_signature_alter(0, step), 0);
        }
    }

    #[test]
    fn three_flats_flattens_b_e_a_only() {
        // fifths = -3 → B♭, E♭, A♭.
        for step in ['B', 'E', 'A'] {
            assert_eq!(key_signature_alter(-3, step), -1, "step {step}");
        }
        for step in ['C', 'D', 'F', 'G'] {
            assert_eq!(key_signature_alter(-3, step), 0, "step {step}");
        }
    }

    #[test]
    fn two_sharps_sharpens_f_c_only() {
        // fifths = 2 → F♯, C♯.
        assert_eq!(key_signature_alter(2, 'F'), 1);
        assert_eq!(key_signature_alter(2, 'C'), 1);
        for step in ['G', 'D', 'A', 'E', 'B'] {
            assert_eq!(key_signature_alter(2, step), 0, "step {step}");
        }
    }

    #[test]
    fn fifths_are_clamped_to_seven_without_panic() {
        // Beyond ±7 no further distinct steps exist; every step is altered.
        for step in ['A', 'B', 'C', 'D', 'E', 'F', 'G'] {
            assert_eq!(key_signature_alter(10, step), 1, "sharp step {step}");
            assert_eq!(key_signature_alter(-10, step), -1, "flat step {step}");
        }
    }

    #[test]
    fn step_is_case_insensitive() {
        assert_eq!(key_signature_alter(-3, 'b'), -1);
        assert_eq!(key_signature_alter(2, 'f'), 1);
    }
}
