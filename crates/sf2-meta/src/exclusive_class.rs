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

//! Splitting the exclusive classes a **stereo** region pair shares (change:
//! add-drum-audio-channel — beta fix).
//!
//! ## The defect this works around
//!
//! A SoundFont "exclusive class" is a choke group: striking one member cuts the
//! others (the closed hi-hat killing the open one is the canonical case).
//! `rustysynth` implements it in `VoiceCollection::request_new` by **reusing the
//! voice slot** of any sounding voice that carries the same class — including a
//! voice started microseconds earlier by the *same* note-on.
//!
//! A stereo piece is two mono regions hard-panned left and right (FluidR3
//! writes every hi-hat that way), and both carry the group's class. So the
//! note-on starts the left half, then the right half takes its slot back: the
//! piece comes out **hard right at half its level** — and completely silent on
//! a mono output, which is exactly what a beta tester heard from the hi-hat
//! while the pad still lit up.
//!
//! ## The fix
//!
//! Give each additional region of one note-on its own class, *deterministically*:
//! the k-th overlapping member of class `c` becomes `c + k * stride`, where
//! `stride` is past every class the file already uses, so a fresh class can
//! never collide with a real one. Both halves then take a voice of their own —
//! and the choke survives, because the next stroke's left half still lands on
//! the left half's class and its right half on the right's.
//!
//! Pure and allocation-light: it reads the `pdta` hydra (`inst`/`ibag`/`igen`)
//! and returns byte patches. Sample data is never touched.

/// A two-byte little-endian patch to apply to the `pdta` LIST body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenPatch {
    /// Offset of the generator's *amount* word, relative to the `pdta` body.
    pub offset: usize,
    /// The exclusive class to write there.
    pub value: u16,
}

/// SoundFont generator operators this module reads.
const GEN_KEY_RANGE: u16 = 43;
const GEN_VEL_RANGE: u16 = 44;
const GEN_SAMPLE_ID: u16 = 53;
const GEN_EXCLUSIVE_CLASS: u16 = 57;

const INST_RECORD: usize = 22;
const BAG_RECORD: usize = 4;
const GEN_RECORD: usize = 4;

/// One instrument zone, reduced to what a choke decision needs.
#[derive(Clone, Copy)]
struct Zone {
    key: (u8, u8),
    vel: (u8, u8),
    /// The zone's own exclusive class and the offset of its amount word.
    exclusive: Option<(u16, usize)>,
    /// Whether the zone names a sample — a zone that does not is the
    /// instrument's *global* zone, which only supplies defaults.
    has_sample: bool,
}

/// The patches that stop the overlapping regions of one note-on from sharing an
/// exclusive class, given the `pdta` LIST **body** (the bytes after the `pdta`
/// form type). Empty when the font has no exclusive classes, when nothing
/// overlaps, or when the hydra is unreadable — never an error: a font we cannot
/// analyse is simply loaded as it is.
pub fn stereo_exclusive_class_patches(pdta: &[u8]) -> Vec<GenPatch> {
    let (Some(inst), Some(ibag), Some((igen, igen_at))) = (
        sub_chunk(pdta, b"inst").map(|(b, _)| b),
        sub_chunk(pdta, b"ibag").map(|(b, _)| b),
        sub_chunk(pdta, b"igen"),
    ) else {
        return Vec::new();
    };

    // A fresh class must be outside the file's own range, so a split half can
    // never be choked by an unrelated piece that happens to use that number.
    let stride = match max_exclusive_class(igen) {
        Some(max) => max as u32 + 1,
        None => return Vec::new(), // no choke groups at all: nothing to split
    };

    let mut patches = Vec::new();
    let instruments = inst.len() / INST_RECORD;
    for i in 0..instruments.saturating_sub(1) {
        let first = bag_index(inst, i);
        let last = bag_index(inst, i + 1);
        split_instrument(ibag, igen, igen_at, first, last, stride, &mut patches);
    }
    patches
}

/// A sounding zone that has already taken a slot in its choke group.
struct Placed {
    key: (u8, u8),
    vel: (u8, u8),
    /// The class the file wrote — what a later zone's group membership is
    /// decided on, never the split value it may have been given.
    class: u16,
}

/// Splits one instrument's zones, appending its patches to `patches`.
fn split_instrument(
    ibag: &[u8],
    igen: &[u8],
    igen_at: usize,
    first_bag: usize,
    last_bag: usize,
    stride: u32,
    patches: &mut Vec<GenPatch>,
) {
    let mut defaults = Zone {
        key: (0, 127),
        vel: (0, 127),
        exclusive: None,
        has_sample: false,
    };
    // Zones already placed, each keeping the class as the FILE wrote it — a
    // later zone counts the members of its own group that already took a slot,
    // so the third overlapping region lands past the second rather than on it.
    let mut placed: Vec<Placed> = Vec::new();

    for bag in first_bag..last_bag {
        let (gen_first, gen_last) = match gen_span(ibag, bag) {
            Some(span) => span,
            None => return,
        };
        let zone = read_zone(igen, igen_at, gen_first, gen_last, &defaults);
        if !zone.has_sample {
            // The instrument's global zone: its generators are the defaults for
            // every zone after it, and it sounds nothing itself.
            defaults = Zone {
                has_sample: false,
                ..zone
            };
            continue;
        }
        let Some((class, at)) = zone.exclusive else {
            continue;
        };
        // How many earlier zones of this note-on already hold this class.
        let taken = placed
            .iter()
            .filter(|p| p.class == class && overlaps(p.key, zone.key) && overlaps(p.vel, zone.vel))
            .count();
        let split = class as u32 + taken as u32 * stride;
        // Out of range for the 16-bit generator amount: leave the zone alone
        // (a shared class costs a channel, a wrong one costs a wrong choke).
        let Ok(split) = u16::try_from(split) else {
            continue;
        };
        if split != class {
            patches.push(GenPatch {
                offset: at,
                value: split,
            });
        }
        placed.push(Placed {
            key: zone.key,
            vel: zone.vel,
            class,
        });
    }
}

/// Reads one zone's generators, falling back to the instrument's `defaults`.
fn read_zone(igen: &[u8], igen_at: usize, first: usize, last: usize, defaults: &Zone) -> Zone {
    let mut zone = Zone {
        key: defaults.key,
        vel: defaults.vel,
        exclusive: defaults.exclusive.map(|(class, _)| (class, usize::MAX)),
        has_sample: false,
    };
    for g in first..last {
        let at = g * GEN_RECORD;
        let (Some(oper), Some(amount)) = (word(igen, at), word(igen, at + 2)) else {
            break;
        };
        match oper {
            GEN_KEY_RANGE => zone.key = (amount as u8, (amount >> 8) as u8),
            GEN_VEL_RANGE => zone.vel = (amount as u8, (amount >> 8) as u8),
            GEN_SAMPLE_ID => zone.has_sample = true,
            GEN_EXCLUSIVE_CLASS => zone.exclusive = Some((amount, igen_at + at + 2)),
            _ => {}
        }
    }
    // A class inherited from the global zone has no amount word of its own in
    // this zone, so it cannot be patched here — treat it as unsplittable.
    if matches!(zone.exclusive, Some((_, usize::MAX))) {
        zone.exclusive = None;
    }
    zone
}

/// The highest exclusive class the generator list declares, if any.
fn max_exclusive_class(igen: &[u8]) -> Option<u16> {
    let mut max = None;
    for g in 0..igen.len() / GEN_RECORD {
        let at = g * GEN_RECORD;
        if word(igen, at) == Some(GEN_EXCLUSIVE_CLASS) {
            let amount = word(igen, at + 2)?;
            if amount != 0 {
                max = Some(max.map_or(amount, |m: u16| m.max(amount)));
            }
        }
    }
    max
}

/// Inclusive-range intersection, the SoundFont zone-selection rule.
fn overlaps(a: (u8, u8), b: (u8, u8)) -> bool {
    a.0 <= b.1 && b.0 <= a.1
}

/// The generator span `[first, last)` of bag index `bag`.
fn gen_span(ibag: &[u8], bag: usize) -> Option<(usize, usize)> {
    let first = word(ibag, bag * BAG_RECORD)? as usize;
    let last = word(ibag, (bag + 1) * BAG_RECORD)? as usize;
    (first <= last).then_some((first, last))
}

/// The first bag index of instrument record `i`.
fn bag_index(inst: &[u8], i: usize) -> usize {
    word(inst, i * INST_RECORD + 20).unwrap_or(0) as usize
}

fn word(bytes: &[u8], at: usize) -> Option<u16> {
    let raw = bytes.get(at..at + 2)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}

/// The body of `id` inside a `pdta` LIST body, with its offset in that body.
fn sub_chunk<'a>(pdta: &'a [u8], id: &[u8; 4]) -> Option<(&'a [u8], usize)> {
    let mut at = 0usize;
    while at + 8 <= pdta.len() {
        let size = u32::from_le_bytes(pdta.get(at + 4..at + 8)?.try_into().ok()?) as usize;
        let body = at + 8;
        if &pdta[at..at + 4] == id {
            let end = body.checked_add(size)?.min(pdta.len());
            return Some((pdta.get(body..end)?, body));
        }
        at = body + size + (size & 1);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A generator record.
    fn generator(oper: u16, amount: u16) -> Vec<u8> {
        let mut out = oper.to_le_bytes().to_vec();
        out.extend_from_slice(&amount.to_le_bytes());
        out
    }

    fn range(lo: u8, hi: u8) -> u16 {
        u16::from(lo) | (u16::from(hi) << 8)
    }

    fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = id.to_vec();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(body);
        if body.len() % 2 == 1 {
            out.push(0);
        }
        out
    }

    /// One instrument whose zones are the given generator lists, wrapped in a
    /// `pdta` body (`inst` + `ibag` + `igen`, in the spec's order).
    fn pdta_of(zones: &[Vec<u8>]) -> Vec<u8> {
        let mut igen = Vec::new();
        let mut ibag = Vec::new();
        let mut index = 0u16;
        for zone in zones {
            ibag.extend_from_slice(&index.to_le_bytes());
            ibag.extend_from_slice(&0u16.to_le_bytes()); // no modulators
            igen.extend_from_slice(zone);
            index += (zone.len() / GEN_RECORD) as u16;
        }
        // Terminal bag record, so the last zone has an end index.
        ibag.extend_from_slice(&index.to_le_bytes());
        ibag.extend_from_slice(&0u16.to_le_bytes());

        let mut inst = vec![0u8; INST_RECORD]; // the instrument, bag index 0
        inst[..3].copy_from_slice(b"Kit");
        inst.extend_from_slice(&[0u8; INST_RECORD]); // EOI terminal
        let eoi = inst.len() - 2;
        inst[eoi..].copy_from_slice(&(zones.len() as u16).to_le_bytes());

        let mut pdta = chunk(b"inst", &inst);
        pdta.extend_from_slice(&chunk(b"ibag", &ibag));
        pdta.extend_from_slice(&chunk(b"igen", &igen));
        pdta
    }

    /// A sounding zone: key range, exclusive class, sample.
    fn stereo_zone(key: (u8, u8), class: u16, sample: u16) -> Vec<u8> {
        let mut out = generator(GEN_KEY_RANGE, range(key.0, key.1));
        out.extend_from_slice(&generator(GEN_EXCLUSIVE_CLASS, class));
        out.extend_from_slice(&generator(GEN_SAMPLE_ID, sample));
        out
    }

    /// Applies `patches` and reads back the class of every zone, in order.
    fn classes_after(pdta: &[u8], patches: &[GenPatch]) -> Vec<u16> {
        let mut bytes = pdta.to_vec();
        for patch in patches {
            bytes[patch.offset..patch.offset + 2].copy_from_slice(&patch.value.to_le_bytes());
        }
        let (igen, _) = sub_chunk(&bytes, b"igen").expect("igen");
        (0..igen.len() / GEN_RECORD)
            .filter(|g| word(igen, g * GEN_RECORD) == Some(GEN_EXCLUSIVE_CLASS))
            .filter_map(|g| word(igen, g * GEN_RECORD + 2))
            .collect()
    }

    #[test]
    fn the_two_halves_of_a_stereo_piece_stop_sharing_one_class() {
        // The defect in miniature: one key, two hard-panned regions, one class.
        let pdta = pdta_of(&[stereo_zone((42, 42), 1, 0), stereo_zone((42, 42), 1, 1)]);
        let patches = stereo_exclusive_class_patches(&pdta);
        assert_eq!(patches.len(), 1, "only the second half moves");
        let classes = classes_after(&pdta, &patches);
        assert_eq!(classes[0], 1, "the first half keeps the file's own class");
        assert_ne!(classes[1], classes[0]);
        assert!(classes[1] > 1, "a fresh class sits past every real one");
    }

    #[test]
    fn the_choke_group_survives_the_split() {
        // Closed (42) and open (46) hi-hat, each in stereo, all class 1: the
        // halves must split *consistently*, so the closed left half still
        // shares a class with the open left half — that is the choke.
        let pdta = pdta_of(&[
            stereo_zone((42, 42), 1, 0),
            stereo_zone((42, 42), 1, 1),
            stereo_zone((46, 46), 1, 2),
            stereo_zone((46, 46), 1, 3),
        ]);
        let classes = classes_after(&pdta, &stereo_exclusive_class_patches(&pdta));
        assert_eq!(classes[0], classes[2], "left halves choke each other");
        assert_eq!(classes[1], classes[3], "right halves choke each other");
        assert_ne!(classes[0], classes[1]);
    }

    #[test]
    fn pieces_that_do_not_overlap_keep_the_class_they_were_given() {
        // Different keys of one choke group are not one note-on: they must NOT
        // be split, or the group would stop choking.
        let pdta = pdta_of(&[stereo_zone((42, 42), 1, 0), stereo_zone((46, 46), 1, 1)]);
        assert!(stereo_exclusive_class_patches(&pdta).is_empty());
    }

    #[test]
    fn velocity_layers_of_one_key_are_one_note_on_only_where_they_overlap() {
        let soft = {
            let mut z = stereo_zone((36, 36), 2, 0);
            z.extend_from_slice(&generator(GEN_VEL_RANGE, range(0, 80)));
            z
        };
        let loud = {
            let mut z = stereo_zone((36, 36), 2, 1);
            z.extend_from_slice(&generator(GEN_VEL_RANGE, range(81, 127)));
            z
        };
        // Disjoint velocity bands never sound together: nothing to split.
        assert!(stereo_exclusive_class_patches(&pdta_of(&[soft.clone(), loud])).is_empty());
        // The same band twice (a stereo pair inside one layer) does split.
        assert_eq!(
            stereo_exclusive_class_patches(&pdta_of(&[soft.clone(), soft])).len(),
            1
        );
    }

    #[test]
    fn a_global_zone_supplies_defaults_and_is_never_itself_a_region() {
        // The instrument's global zone (no sample) sets the key range every
        // following zone inherits — so two sample zones with no range of their
        // own still read as overlapping.
        let global = generator(GEN_KEY_RANGE, range(42, 42));
        let mut left = generator(GEN_EXCLUSIVE_CLASS, 1);
        left.extend_from_slice(&generator(GEN_SAMPLE_ID, 0));
        let mut right = generator(GEN_EXCLUSIVE_CLASS, 1);
        right.extend_from_slice(&generator(GEN_SAMPLE_ID, 1));
        let pdta = pdta_of(&[global, left, right]);
        assert_eq!(stereo_exclusive_class_patches(&pdta).len(), 1);
    }

    #[test]
    fn a_font_with_no_choke_groups_is_left_alone() {
        let mut zone = generator(GEN_KEY_RANGE, range(0, 127));
        zone.extend_from_slice(&generator(GEN_SAMPLE_ID, 0));
        assert!(stereo_exclusive_class_patches(&pdta_of(&[zone.clone(), zone])).is_empty());
        // …and so is anything that is not a readable hydra.
        assert!(stereo_exclusive_class_patches(b"not a pdta body").is_empty());
        assert!(stereo_exclusive_class_patches(&[]).is_empty());
    }

    #[test]
    fn a_third_overlapping_region_takes_a_third_class() {
        let pdta = pdta_of(&[
            stereo_zone((42, 42), 1, 0),
            stereo_zone((42, 42), 1, 1),
            stereo_zone((42, 42), 1, 2),
        ]);
        let classes = classes_after(&pdta, &stereo_exclusive_class_patches(&pdta));
        assert_eq!(classes.len(), 3);
        assert_ne!(classes[0], classes[1]);
        assert_ne!(classes[1], classes[2]);
        assert_ne!(classes[0], classes[2]);
    }
}
