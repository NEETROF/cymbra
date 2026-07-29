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

//! Pure logic for editing a catalog score's curatorial metadata (change:
//! add-catalog-metadata-editing).
//!
//! [`plan_edit`] resolves an incoming change set against the row's current values:
//! it validates (level enum; a title is mandatory), computes the per-field diff for
//! the audit trail, and **recomputes the derived search keys** (`title_norm`,
//! `composer_norm`, `work_key`) from the *final* title/composer via the same
//! `normalize_text` the ingest uses — so search and same-work grouping follow the
//! edit rather than desyncing. No IO here; the repo applies the [`EditPlan`] in one
//! transaction.

use cymbra_platform::{AppError, Result};

/// The difficulty levels a moderator may set (matching the DB CHECK).
const LEVELS: [&str; 3] = ["beginner", "intermediate", "advanced"];

/// A catalog row's current curatorial values (for diffing).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CurrentMeta {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub arranger: Option<String>,
    pub level: Option<String>,
}

/// Requested changes: `None` = leave the field unchanged; `Some(v)` = set it (an empty
/// string clears composer/arranger/level; an empty title is rejected — a title is
/// mandatory).
#[derive(Debug, Clone, Default)]
pub struct MetadataChanges {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub arranger: Option<String>,
    pub level: Option<String>,
}

/// One field's before→after, recorded in the audit trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    pub field: &'static str,
    pub old: Option<String>,
    pub new: Option<String>,
}

/// The resolved edit: the final curatorial values, the recomputed derived search keys,
/// and the per-field diff (empty ⇒ nothing changed, a no-op).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditPlan {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub arranger: Option<String>,
    pub level: Option<String>,
    pub title_norm: Option<String>,
    pub composer_norm: String,
    pub work_key: String,
    pub changes: Vec<FieldChange>,
}

impl EditPlan {
    /// Whether the level field changed (so the row's `level_source` becomes `manual`).
    pub fn level_changed(&self) -> bool {
        self.changes.iter().any(|c| c.field == "level")
    }
}

/// Trim an incoming value; an empty result clears the field (`None`).
fn cleaned(v: &str) -> Option<String> {
    let t = v.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Resolve `changes` against `current` into an [`EditPlan`]. Validates the level enum
/// and a non-empty title; recomputes the derived search keys from the final values.
pub fn plan_edit(current: &CurrentMeta, changes: &MetadataChanges) -> Result<EditPlan> {
    let title = match &changes.title {
        None => current.title.clone(),
        Some(v) => match cleaned(v) {
            None => return Err(AppError::InvalidArgument("a title is required".into())),
            some => some,
        },
    };
    let composer = match &changes.composer {
        None => current.composer.clone(),
        Some(v) => cleaned(v),
    };
    let arranger = match &changes.arranger {
        None => current.arranger.clone(),
        Some(v) => cleaned(v),
    };
    let level = match &changes.level {
        None => current.level.clone(),
        Some(v) => {
            let c = cleaned(v);
            if let Some(l) = c.as_deref()
                && !LEVELS.contains(&l)
            {
                return Err(AppError::InvalidArgument(format!("unknown level {l:?}")));
            }
            c
        }
    };

    let mut ch: Vec<FieldChange> = Vec::new();
    diff(&mut ch, "title", &current.title, &title);
    diff(&mut ch, "composer", &current.composer, &composer);
    diff(&mut ch, "arranger", &current.arranger, &arranger);
    diff(&mut ch, "level", &current.level, &level);

    // Derived search keys from the FINAL title/composer, via the same normalization the
    // ingest/backfill use (so the trigram index + work_key grouping stay consistent).
    let title_norm = title.as_deref().map(cymbra_musicxml_core::normalize_text);
    let composer_norm = composer
        .as_deref()
        .map(cymbra_musicxml_core::normalize_text)
        .unwrap_or_default();
    let work_key = format!(
        "{}::{}",
        composer_norm,
        title_norm.clone().unwrap_or_default()
    );

    Ok(EditPlan {
        title,
        composer,
        arranger,
        level,
        title_norm,
        composer_norm,
        work_key,
        changes: ch,
    })
}

fn diff(
    out: &mut Vec<FieldChange>,
    field: &'static str,
    old: &Option<String>,
    new: &Option<String>,
) {
    if old != new {
        out.push(FieldChange {
            field,
            old: old.clone(),
            new: new.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current() -> CurrentMeta {
        CurrentMeta {
            title: Some("Clair de Lune".into()),
            composer: Some("Debussy".into()),
            arranger: None,
            level: Some("advanced".into()),
        }
    }

    #[test]
    fn edits_composer_and_recomputes_keys() {
        let changes = MetadataChanges {
            composer: Some("Claude Debussy".into()),
            ..Default::default()
        };
        let plan = plan_edit(&current(), &changes).unwrap();
        assert_eq!(plan.composer.as_deref(), Some("Claude Debussy"));
        assert_eq!(plan.composer_norm, "claude debussy");
        assert_eq!(plan.work_key, "claude debussy::clair de lune");
        // Only composer changed → one audit entry.
        assert_eq!(plan.changes.len(), 1);
        assert_eq!(plan.changes[0].field, "composer");
        assert_eq!(plan.changes[0].old.as_deref(), Some("Debussy"));
        assert_eq!(plan.changes[0].new.as_deref(), Some("Claude Debussy"));
    }

    #[test]
    fn absent_fields_are_unchanged_noop_has_no_changes() {
        // Nothing supplied → no changes at all.
        let plan = plan_edit(&current(), &MetadataChanges::default()).unwrap();
        assert!(plan.changes.is_empty());
        // Setting the same value → still a no-op (diff, not overwrite).
        let same = MetadataChanges {
            title: Some("Clair de Lune".into()),
            ..Default::default()
        };
        assert!(plan_edit(&current(), &same).unwrap().changes.is_empty());
    }

    #[test]
    fn empty_composer_clears_it_and_recomputes() {
        let changes = MetadataChanges {
            composer: Some("  ".into()),
            ..Default::default()
        };
        let plan = plan_edit(&current(), &changes).unwrap();
        assert_eq!(plan.composer, None);
        assert_eq!(plan.composer_norm, "");
        assert_eq!(plan.work_key, "::clair de lune");
        assert_eq!(plan.changes.len(), 1);
        assert_eq!(plan.changes[0].new, None);
    }

    #[test]
    fn empty_title_is_rejected() {
        let changes = MetadataChanges {
            title: Some("   ".into()),
            ..Default::default()
        };
        assert!(matches!(
            plan_edit(&current(), &changes),
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[test]
    fn invalid_level_is_rejected_and_valid_marks_level_changed() {
        let bad = MetadataChanges {
            level: Some("expert".into()),
            ..Default::default()
        };
        assert!(matches!(
            plan_edit(&current(), &bad),
            Err(AppError::InvalidArgument(_))
        ));

        let ok = MetadataChanges {
            level: Some("beginner".into()),
            ..Default::default()
        };
        let plan = plan_edit(&current(), &ok).unwrap();
        assert_eq!(plan.level.as_deref(), Some("beginner"));
        assert!(plan.level_changed());
    }

    #[test]
    fn clearing_level_with_empty_string() {
        let changes = MetadataChanges {
            level: Some("".into()),
            ..Default::default()
        };
        let plan = plan_edit(&current(), &changes).unwrap();
        assert_eq!(plan.level, None);
        assert!(plan.level_changed());
    }
}
