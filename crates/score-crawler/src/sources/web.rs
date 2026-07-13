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

//! Pure HTML parsing helpers shared by the web-crawl adapters.
//!
//! Given a page's HTML (fetched via a [`crate::http::Fetcher`]), these extract
//! links and detect a licence signal without any I/O, so each adapter's parsing
//! is unit-tested against a fixture page. No network here.

use anyhow::{Result, anyhow};
use scraper::{Html, Selector};

/// All `href` values on the page, resolved against `base` when relative.
pub fn links(html: &str, base: &str) -> Result<Vec<String>> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("a[href]").map_err(|e| anyhow!("bad selector: {e}"))?;
    let base = reqwest::Url::parse(base).ok();
    Ok(doc
        .select(&sel)
        .filter_map(|e| e.value().attr("href"))
        .filter_map(|href| resolve(base.as_ref(), href))
        .collect())
}

/// Links whose path ends in one of `exts` (case-insensitive, no leading dot),
/// e.g. `["musicxml", "mxl"]`.
pub fn links_with_ext(html: &str, base: &str, exts: &[&str]) -> Result<Vec<String>> {
    let all = links(html, base)?;
    Ok(all
        .into_iter()
        .filter(|u| {
            let path = u.split(['?', '#']).next().unwrap_or(u).to_ascii_lowercase();
            exts.iter().any(|e| path.ends_with(&format!(".{e}")))
        })
        .collect())
}

/// Detects a licence signal on the page: the first Creative Commons / public
/// domain deed link, else a "public domain" mention in the text. Returns the raw
/// signal for [`crate::license::normalize`], or `None` if nothing is found (the
/// gate then rejects it as unknown).
pub fn detect_license(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    if let Ok(sel) = Selector::parse("a[href]") {
        for e in doc.select(&sel) {
            if let Some(href) = e.value().attr("href") {
                let low = href.to_ascii_lowercase();
                if low.contains("creativecommons.org/licenses")
                    || low.contains("creativecommons.org/publicdomain")
                {
                    return Some(href.to_string());
                }
            }
        }
    }
    let text = doc.root_element().text().collect::<String>().to_lowercase();
    if text.contains("public domain") {
        return Some("Public Domain".to_string());
    }
    // A stated "all rights reserved" is a real (rejecting) signal — surface it so
    // the gate records a licence rejection rather than an unknown-licence failure.
    if text.contains("all rights reserved") {
        return Some("All Rights Reserved".to_string());
    }
    None
}

/// Resolves `href` against an optional base URL; drops unparseable results.
fn resolve(base: Option<&reqwest::Url>, href: &str) -> Option<String> {
    if let Ok(abs) = reqwest::Url::parse(href) {
        return Some(abs.to_string());
    }
    base.and_then(|b| b.join(href).ok()).map(|u| u.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = r#"<html><body>
      <a href="/scores/one.musicxml">One</a>
      <a href="two.mxl">Two</a>
      <a href="https://other.org/three.pdf">PDF</a>
      <a href="https://creativecommons.org/licenses/by-sa/4.0/">licence</a>
    </body></html>"#;

    #[test]
    fn resolves_relative_and_absolute_links() {
        let ls = links(PAGE, "https://cpdl.org/wiki/page").unwrap();
        assert!(ls.contains(&"https://cpdl.org/scores/one.musicxml".to_string()));
        assert!(ls.contains(&"https://cpdl.org/wiki/two.mxl".to_string()));
        assert!(ls.contains(&"https://other.org/three.pdf".to_string()));
    }

    #[test]
    fn filters_by_score_extension() {
        let ls = links_with_ext(PAGE, "https://cpdl.org/wiki/page", &["musicxml", "mxl"]).unwrap();
        assert_eq!(ls.len(), 2);
        assert!(
            ls.iter()
                .all(|u| u.ends_with(".musicxml") || u.ends_with(".mxl"))
        );
    }

    #[test]
    fn detects_cc_license_link() {
        assert_eq!(
            detect_license(PAGE).as_deref(),
            Some("https://creativecommons.org/licenses/by-sa/4.0/")
        );
    }

    #[test]
    fn detects_public_domain_text() {
        let html = "<html><body>This work is in the Public Domain.</body></html>";
        assert_eq!(detect_license(html).as_deref(), Some("Public Domain"));
    }

    #[test]
    fn returns_none_when_no_license() {
        let html = "<html><body>All rights unknown here</body></html>";
        assert_eq!(detect_license(html), None);
    }
}
