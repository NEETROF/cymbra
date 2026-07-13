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

//! The adapter registry: source name → concrete [`SourceAdapter`].
//!
//! Maps the enabled source names to adapter instances (git-clone or web-crawl).
//! Sources whose adapter is not implemented yet are returned as `unsupported`
//! so the caller can log them rather than silently ignoring coverage gaps.

use std::path::Path;
use std::sync::Arc;

use crate::http::Fetcher;
use crate::sources::SourceAdapter;
use crate::sources::git::GitRepoSource;
use crate::sources::pdmx::PdmxSource;
use crate::sources::web_index::WebIndexSource;

/// Default CPDL listing (a scores category page).
pub const CPDL_LISTING: &str = "https://www.cpdl.org/wiki/index.php/Category:Scores";
/// Default IMSLP listing.
pub const IMSLP_LISTING: &str = "https://imslp.org/wiki/Category:Scores";
/// Default Project Gutenberg sheet-music listing.
pub const GUTENBERG_LISTING: &str = "https://www.gutenberg.org/ebooks/subject/2955";
/// Default Hymnary listing.
pub const HYMNARY_LISTING: &str = "https://hymnary.org/browse/tunes";
/// Default PDMX (Zenodo) metadata index.
pub const PDMX_METADATA: &str = "https://zenodo.org/records/15571083/files/metadata.json";

/// The adapters built for a set of source names, plus the names not yet wired.
pub struct BuiltAdapters {
    pub adapters: Vec<Box<dyn SourceAdapter>>,
    pub unsupported: Vec<String>,
}

/// Builds adapters for `sources`. Git adapters clone under
/// `checkout_root/<name>`; web adapters use the shared `fetcher`.
pub fn build_adapters(
    sources: &[String],
    fetcher: Arc<dyn Fetcher>,
    checkout_root: &Path,
) -> BuiltAdapters {
    let mut adapters: Vec<Box<dyn SourceAdapter>> = Vec::new();
    let mut unsupported = Vec::new();

    for name in sources {
        let adapter: Option<Box<dyn SourceAdapter>> = match name.as_str() {
            "openscore" => Some(Box::new(GitRepoSource::openscore(
                checkout_root.join("openscore"),
            ))),
            "musetrainer" => Some(Box::new(GitRepoSource::musetrainer(
                checkout_root.join("musetrainer"),
            ))),
            "eduardomourar" => Some(Box::new(GitRepoSource::eduardomourar(
                checkout_root.join("eduardomourar"),
            ))),
            "cpdl" => Some(Box::new(WebIndexSource::cpdl(
                fetcher.clone(),
                CPDL_LISTING,
            ))),
            "imslp" => Some(Box::new(WebIndexSource::imslp(
                fetcher.clone(),
                IMSLP_LISTING,
            ))),
            "gutenberg" => Some(Box::new(WebIndexSource::gutenberg(
                fetcher.clone(),
                GUTENBERG_LISTING,
            ))),
            "hymnary" => Some(Box::new(WebIndexSource::hymnary(
                fetcher.clone(),
                HYMNARY_LISTING,
            ))),
            "pdmx" => Some(Box::new(PdmxSource::new(fetcher.clone(), PDMX_METADATA))),
            _ => None,
        };
        match adapter {
            Some(a) => adapters.push(a),
            None => unsupported.push(name.clone()),
        }
    }
    BuiltAdapters {
        adapters,
        unsupported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::test_fetcher::MapFetcher;

    #[test]
    fn builds_known_sources_and_flags_the_rest() {
        let fetcher: Arc<dyn Fetcher> = Arc::new(MapFetcher::default());
        let sources = [
            "openscore".to_string(),
            "cpdl".to_string(),
            "pdmx".to_string(),
            "imslp".to_string(),
            "mutopia".to_string(), // not wired yet (LilyPond)
        ];
        let built = build_adapters(&sources, fetcher, Path::new("/tmp/checkouts"));
        assert_eq!(built.adapters.len(), 4);
        assert_eq!(built.unsupported, vec!["mutopia"]);
        let names: Vec<&str> = built.adapters.iter().map(|a| a.name()).collect();
        assert!(
            names.contains(&"openscore") && names.contains(&"cpdl") && names.contains(&"imslp")
        );
    }
}
