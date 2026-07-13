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
//! Maps the enabled source names to adapter instances. Every current source is
//! either a git clone (`openscore`, `musetrainer`, `eduardomourar`, `mutopia`)
//! or a bulk dataset (`pdmx`) — the web-crawl family was removed (see
//! [`crate::sources::EXCLUDED_SOURCES`]). Names with no adapter are returned as
//! `unsupported` so the caller can log them rather than silently dropping them.

use std::path::Path;

use crate::sources::SourceAdapter;
use crate::sources::git::GitRepoSource;
use crate::sources::mutopia::MutopiaSource;
use crate::sources::pdmx::PdmxDatasetSource;

/// The adapters built for a set of source names, plus the names not wired.
pub struct BuiltAdapters {
    pub adapters: Vec<Box<dyn SourceAdapter>>,
    pub unsupported: Vec<String>,
}

/// Builds adapters for `sources`. Git adapters clone under
/// `checkout_root/<name>`; the PDMX dataset caches there too.
pub fn build_adapters(sources: &[String], checkout_root: &Path) -> BuiltAdapters {
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
            "mutopia" => Some(Box::new(MutopiaSource::new(checkout_root.join("mutopia")))),
            "pdmx" => Some(Box::new(PdmxDatasetSource::new(checkout_root.join("pdmx")))),
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

    #[test]
    fn builds_known_sources_and_flags_the_rest() {
        let sources = [
            "openscore".to_string(),
            "mutopia".to_string(),
            "pdmx".to_string(),
            "musetrainer".to_string(),
            "bogus".to_string(), // unknown source name → reported, not built
        ];
        let built = build_adapters(&sources, Path::new("/tmp/checkouts"));
        assert_eq!(built.adapters.len(), 4);
        assert_eq!(built.unsupported, vec!["bogus"]);
        let names: Vec<&str> = built.adapters.iter().map(|a| a.name()).collect();
        assert!(names.contains(&"openscore") && names.contains(&"pdmx"));
    }

    #[test]
    fn excluded_sources_are_not_buildable() {
        // The dropped web sources must not silently resolve to an adapter.
        for (name, _reason) in crate::sources::EXCLUDED_SOURCES {
            let built = build_adapters(&[(*name).to_string()], Path::new("/tmp/checkouts"));
            assert!(built.adapters.is_empty(), "{name} must not build");
            assert_eq!(built.unsupported, vec![name.to_string()]);
        }
    }
}
