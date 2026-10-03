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

//! The calls the engine refuses (`generalise-lingua-wasm-engine`). They throw, which
//! only a wasm target can show: creating a `JsError` off wasm panics.
//! `wasm-pack test --node crates/lingua-wasm`.

#![cfg(target_arch = "wasm32")]

use lingua_wasm::LinguaEngine;
use wasm_bindgen_test::wasm_bindgen_test;

const PACK: &[u8] = include_bytes!("fixtures/pack.lingua");

#[wasm_bindgen_test]
fn spec_scenario_a_language_without_a_pack_is_refused() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    assert!(
        engine
            .analyse(vec!["Hola".to_owned()], Some("es".to_owned()))
            .is_err()
    );
    assert!(
        engine
            .set_status("haber", "known", Some("es".to_owned()))
            .is_err()
    );
    assert!(
        engine
            .analyse(vec!["Olá".to_owned()], Some("pt".to_owned()))
            .is_err()
    );
    assert_eq!(
        engine.export_status_ops(),
        "[]",
        "a refused call changes nothing"
    );
}

#[wasm_bindgen_test]
fn spec_scenario_an_invalid_choice_of_studied_languages_is_refused() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    engine
        .set_studied_languages(vec!["es".to_owned(), "en".to_owned()])
        .unwrap();
    assert!(engine.set_studied_languages(vec![]).is_err());
    assert!(
        engine
            .set_studied_languages(vec!["en".to_owned(), "en".to_owned()])
            .is_err()
    );
    assert!(engine.set_studied_languages(vec!["pt".to_owned()]).is_err());
    assert_eq!(
        engine.studied_languages(),
        r#"["es","en"]"#,
        "a refused choice leaves the current one"
    );
}

#[wasm_bindgen_test]
fn spec_scenario_one_pack_per_language() {
    let mut engine = LinguaEngine::new(PACK).unwrap();
    assert!(engine.add_pack(PACK).is_err());
    assert_eq!(engine.languages(), r#"["en"]"#);
}
