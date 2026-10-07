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

//! The `lingua` binary: `ingest`, `statusline`, `vocab` and `mcp` subcommands, dispatched
//! for the Claude Code plugin. Thin glue over the library; hooks and the statusline
//! degrade silently so a failure never disrupts the agent. The languages followed are the
//! packs installed (add-lingua-agent-languages D1).

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use lingua_agent::engine::{Library, lingua_home};
use lingua_agent::ingest::run_ingest;
use lingua_agent::mcp;
use lingua_agent::source::{ClaudeCodeSource, SessionSource};
use lingua_agent::statusline::statusline_text;
use lingua_agent::store::Store;
use lingua_agent::vocab::{add_to_deck, listing, skipped_notice, vocab_words};
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::knowledge::profile::NativeLanguage;
use lingua_core::knowledge::state::KnowledgeState;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Read the value following `--flag` in the args.
fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn open_store() -> Option<Store> {
    let home = lingua_home();
    std::fs::create_dir_all(&home).ok()?;
    Store::open(&home.join("lingua.db")).ok()
}

/// The transcript path: `--transcript <path>`, else `transcript_path` from the hook /
/// statusline JSON that Claude Code sends on stdin.
fn resolve_transcript(args: &[String]) -> Option<String> {
    if let Some(path) = flag(args, "--transcript") {
        return Some(path);
    }
    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).ok()?;
    let value: serde_json::Value = serde_json::from_str(&input).ok()?;
    value
        .get("transcript_path")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("");
    match command {
        "ingest" => cmd_ingest(&args),
        "statusline" => cmd_statusline(&args),
        "vocab" => cmd_vocab(&args),
        "mcp" => cmd_mcp(),
        _ => {
            eprintln!(
                "usage: lingua <ingest|statusline|vocab|mcp> [--transcript <path>] [--add w1,w2] [--language <tag>]"
            );
            ExitCode::FAILURE
        }
    }
}

/// Stop hook: ingest new turns. Silent on any failure (never disrupt the agent).
fn cmd_ingest(args: &[String]) -> ExitCode {
    let mut library = Library::installed();
    if library.is_empty() {
        return ExitCode::SUCCESS;
    }
    let (Some(transcript), Some(store)) = (resolve_transcript(args), open_store()) else {
        return ExitCode::SUCCESS;
    };
    let source = ClaudeCodeSource {
        path: PathBuf::from(&transcript),
    };
    let _ = run_ingest(&store, &mut library, &source, &transcript, now());
    ExitCode::SUCCESS
}

/// Statusline: the last reply's coverage. Prints nothing on any failure (mute).
fn cmd_statusline(args: &[String]) -> ExitCode {
    let render = || -> Option<String> {
        let mut library = Library::installed();
        if library.is_empty() {
            return None;
        }
        let transcript = resolve_transcript(args)?;
        let store = open_store()?;
        let (_, texts) = ClaudeCodeSource {
            path: PathBuf::from(transcript),
        }
        .extract(0)
        .ok()?;
        let last = texts.last()?;
        let knowledge = store.knowledge_state().ok()?;
        statusline_text(&mut library, &knowledge, last)
    };
    if let Some(line) = render() {
        println!("{line}");
    }
    ExitCode::SUCCESS
}

/// `/vocab`: list the session's unknown words, or add a selection to the deck.
fn cmd_vocab(args: &[String]) -> ExitCode {
    let Some(transcript) = flag(args, "--transcript") else {
        eprintln!("vocab: --transcript <path> required");
        return ExitCode::FAILURE;
    };
    let mut library = Library::installed();
    let Some(store) = open_store().filter(|_| !library.is_empty()) else {
        println!("Pack de langue introuvable — installe-le dans ~/.lingua/pack.lingua.");
        return ExitCode::SUCCESS;
    };
    let language = match flag(args, "--language") {
        Some(tag) => match StudiedLanguage::from_tag(tag.trim()) {
            Some(language) => Some(language),
            None => {
                eprintln!("vocab: --language {tag} is not a language the plugin knows");
                return ExitCode::FAILURE;
            }
        },
        None => None,
    };
    let Ok((_, texts)) = (ClaudeCodeSource {
        path: PathBuf::from(&transcript),
    })
    .extract(0) else {
        return ExitCode::SUCCESS;
    };
    let knowledge = store
        .knowledge_state()
        .unwrap_or_else(|_| KnowledgeState::new());
    let words = vocab_words(&mut library, &knowledge, &texts, |language| {
        store.calibration(language).unwrap_or(3000)
    });

    if let Some(add) = flag(args, "--add") {
        let chosen: Vec<String> = add
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        let native = library.native().unwrap_or(NativeLanguage::French);
        let Ok(added) = add_to_deck(&store, &words, &chosen, language, native, now()) else {
            return ExitCode::SUCCESS;
        };
        println!("Ajouté {} mot(s) au deck.", added.added);
        if !added.ambiguous.is_empty() {
            println!(
                "Listé(s) dans plusieurs langues, non ajouté(s) : {} — précise --language.",
                added.ambiguous.join(", ")
            );
        }
        return ExitCode::SUCCESS;
    }

    print!("{}", listing(&words, library.several()));
    print!("{}", skipped_notice(library.skipped(), library.native()));
    ExitCode::SUCCESS
}

/// MCP server over stdio (deck operations for the agent).
fn cmd_mcp() -> ExitCode {
    let Some(store) = open_store() else {
        return ExitCode::FAILURE;
    };
    // With no pack installed the deck is still served, as English, as it always was.
    let library = Library::installed();
    let languages = if library.is_empty() {
        vec![StudiedLanguage::English]
    } else {
        library.languages()
    };
    // With no pack, the deck is served as it always was: English, glossed in French.
    let followed = mcp::Followed {
        languages: &languages,
        native: library.native().unwrap_or(NativeLanguage::French),
    };
    match mcp::serve(&store, &followed, now) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
