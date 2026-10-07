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

//! The analysis brain, compiled natively — the same `lingua-core` (same analyser versions) as
//! the extension. Finds the packs installed, which are the languages the plugin follows, reads
//! each reply in its own language and analyses it into classified tokens
//! (add-lingua-agent-languages D1, D2). The plugin follows one native language, `pack.lingua`'s
//! (generalise-lingua-native-language D9). Paths resolve under `~/.lingua/` (overridable by env
//! for tests).

use std::path::PathBuf;

use lingua_core::analysis::language::{StudiedLanguage, detect_document_language};
use lingua_core::engine::{PageAnalysis, analyse_page};
use lingua_core::knowledge::profile::{LanguagePair, NativeLanguage};
use lingua_core::knowledge::state::KnowledgeState;
use lingua_core::packs::Pack;

/// The file name of the English pack, whose native language the plugin follows.
const ANCHOR: &str = "pack.lingua";

/// The plugin's data directory: `$LINGUA_HOME`, else `$HOME/.lingua`.
pub fn lingua_home() -> PathBuf {
    if let Ok(dir) = std::env::var("LINGUA_HOME") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".lingua")
}

/// The English pack's path: `$LINGUA_PACK`, else `<lingua_home>/pack.lingua`.
pub fn pack_path() -> PathBuf {
    std::env::var("LINGUA_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| lingua_home().join("pack.lingua"))
}

/// One language followed: where its pack is, and the pack once loaded.
struct Entry {
    language: StudiedLanguage,
    file: Option<PathBuf>,
    /// `Some(None)` once a load failed, so it is not tried again.
    pack: Option<Option<Pack>>,
}

/// A pack installed but not followed, because it is glossed in another native language than the
/// plugin's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// The file.
    pub file: PathBuf,
    /// The native language it is glossed in.
    pub native: NativeLanguage,
}

/// The packs installed, and the languages they make the plugin follow (design D1). A pack is
/// loaded only when a reply in its language needs it.
pub struct Library {
    entries: Vec<Entry>,
    /// The native language every pack followed is glossed in; `None` with no pack.
    native: Option<NativeLanguage>,
    /// The packs installed but glossed in another native language.
    skipped: Vec<Skipped>,
}

impl Library {
    /// The packs installed: `$LINGUA_PACK` alone when set, else every `*.lingua` file directly in
    /// the data directory, read in file-name order.
    pub fn installed() -> Library {
        if let Ok(path) = std::env::var("LINGUA_PACK") {
            return Library::from_files([PathBuf::from(path)]);
        }
        let mut files: Vec<PathBuf> = std::fs::read_dir(lingua_home())
            .map(|dir| dir.flatten().map(|entry| entry.path()).collect())
            .unwrap_or_default();
        files.retain(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "lingua"));
        files.sort();
        Library::from_files(files)
    }

    /// The packs among `files`, by the pair each says it serves (design D1). A file that is not a
    /// pack this core reads is skipped, and the first pack of a language wins. The plugin follows
    /// one native language (generalise-lingua-native-language D9): `pack.lingua`'s when it reads,
    /// otherwise the first readable file's, in the order given; a pack glossed in another is
    /// skipped and kept in [`Library::skipped`].
    pub fn from_files(files: impl IntoIterator<Item = PathBuf>) -> Library {
        let readable: Vec<(PathBuf, LanguagePair)> = files
            .into_iter()
            .filter_map(|file| {
                let pair = std::fs::read(&file)
                    .ok()
                    .and_then(|bytes| Pack::pair_in(&bytes).ok())?;
                Some((file, pair))
            })
            .collect();
        // Any `xx-en` or `xx-es` name sorts before `pack.lingua`: anchored on file-name order,
        // one stray file would evict the French packs.
        let native = readable
            .iter()
            .find(|(file, _)| file.file_name().is_some_and(|name| name == ANCHOR))
            .or(readable.first())
            .map(|(_, pair)| pair.native);
        let mut entries: Vec<Entry> = Vec::new();
        let mut skipped = Vec::new();
        for (file, pair) in readable {
            if Some(pair.native) != native {
                skipped.push(Skipped {
                    file,
                    native: pair.native,
                });
            } else if entries.iter().all(|entry| entry.language != pair.studied) {
                entries.push(Entry {
                    language: pair.studied,
                    file: Some(file),
                    pack: None,
                });
            }
        }
        Library::sorted(entries, native, skipped)
    }

    /// Packs already loaded, each serving its own language, glossed in the first pack's native
    /// language: a pack glossed in another is left out.
    pub fn from_packs(packs: impl IntoIterator<Item = Pack>) -> Library {
        let mut entries: Vec<Entry> = Vec::new();
        let mut native = None;
        for pack in packs {
            let language = pack.studied();
            if *native.get_or_insert(pack.native()) != pack.native() {
                continue;
            }
            if entries.iter().all(|entry| entry.language != language) {
                entries.push(Entry {
                    language,
                    file: None,
                    pack: Some(Some(pack)),
                });
            }
        }
        Library::sorted(entries, native, Vec::new())
    }

    /// The languages in tag order: the vote's ties go to the earlier, English, as every reply
    /// went before.
    fn sorted(
        mut entries: Vec<Entry>,
        native: Option<NativeLanguage>,
        skipped: Vec<Skipped>,
    ) -> Library {
        entries.sort_by(|a, b| a.language.tag().cmp(b.language.tag()));
        Library {
            entries,
            native,
            skipped,
        }
    }

    /// The native language the packs followed are glossed in; `None` with no pack installed.
    pub fn native(&self) -> Option<NativeLanguage> {
        self.native
    }

    /// The packs installed but not followed, glossed in another native language, in file order.
    pub fn skipped(&self) -> &[Skipped] {
        &self.skipped
    }

    /// The languages followed, in tag order.
    pub fn languages(&self) -> Vec<StudiedLanguage> {
        self.entries.iter().map(|entry| entry.language).collect()
    }

    /// No pack installed: the plugin stays mute.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// More than one language followed: the statusline names the language, the tools ask which.
    pub fn several(&self) -> bool {
        self.entries.len() > 1
    }

    /// The pack of `language`, loaded the first time it is asked for; `None` when that language is
    /// not followed or its pack does not load.
    pub fn pack(&mut self, language: StudiedLanguage) -> Option<&Pack> {
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.language == language)?;
        if entry.pack.is_none() {
            let loaded = entry
                .file
                .as_ref()
                .and_then(|file| std::fs::read(file).ok())
                .and_then(|bytes| Pack::load(&bytes).ok());
            entry.pack = Some(loaded);
        }
        entry.pack.as_ref().and_then(Option::as_ref)
    }

    /// The language a reply is read in among those followed (design D2): the core's vote over its
    /// lines outside fenced code. With one language there is no vote.
    pub fn language_of(&self, text: &str) -> Option<StudiedLanguage> {
        detect_document_language(&prose_lines(text), &self.languages(), None)
    }
}

/// A reply's non-empty lines: the blocks it is analysed in.
pub fn blocks(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .collect()
}

/// The lines of a reply that vote on its language: those outside fenced code blocks, so the code
/// in a Spanish reply does not pull it to English (design D2).
pub fn prose_lines(text: &str) -> Vec<&str> {
    let mut fenced = false;
    let mut prose = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        } else if !fenced && !line.trim().is_empty() {
            prose.push(line);
        }
    }
    prose
}

/// Analyses a reply with `pack`, in the pack's language. Short input, or input in another
/// language, analyses to `analysable: false`.
pub fn analyse(pack: &Pack, knowledge: &KnowledgeState, text: &str) -> PageAnalysis {
    analyse_page(&blocks(text), pack.studied(), pack, knowledge)
}

/// A reply read in its language: that language and the analysis, or `None` when no pack serves it.
pub fn read_reply(
    library: &mut Library,
    knowledge: &KnowledgeState,
    text: &str,
) -> Option<(StudiedLanguage, PageAnalysis)> {
    let language = library.language_of(text)?;
    let pack = library.pack(language)?;
    Some((language, analyse(pack, knowledge, text)))
}
