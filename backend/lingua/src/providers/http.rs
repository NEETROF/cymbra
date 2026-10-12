// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The HTTPS adapters of the translation providers. Thin reqwest glue — coverage-excluded;
//! the bodies and the reading of replies are tested in [`super`]. An error never carries
//! the request, the reply body or the key: only the provider and the status.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use serde_json::Value;

use super::{ANTHROPIC, ProviderConfig, anthropic_body, gemini_body, read_anthropic, read_gemini};
use crate::online_translation::{ProviderReply, TranslationProvider};

const GEMINI_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct HttpProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl HttpProvider {
    pub fn new(config: ProviderConfig) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .connect_timeout(Duration::from_secs(4))
            .build()?;
        Ok(Self { config, client })
    }

    /// The configured providers as ports, primary first.
    pub fn all(configs: Vec<ProviderConfig>) -> anyhow::Result<Vec<Arc<dyn TranslationProvider>>> {
        configs
            .into_iter()
            .map(|c| Ok(Arc::new(Self::new(c)?) as Arc<dyn TranslationProvider>))
            .collect()
    }

    fn unavailable(&self, what: impl std::fmt::Display) -> AppError {
        AppError::Unavailable(format!("{} translation provider: {what}", self.config.name))
    }
}

#[async_trait]
impl TranslationProvider for HttpProvider {
    fn name(&self) -> &'static str {
        self.config.name
    }

    async fn translate(&self, system: &str, user: &str) -> Result<ProviderReply> {
        let request = if self.config.name == ANTHROPIC {
            self.client
                .post(ANTHROPIC_URL)
                .header("x-api-key", &self.config.api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&anthropic_body(&self.config.model, system, user))
        } else {
            self.client
                .post(format!(
                    "{GEMINI_BASE}/{}:generateContent",
                    self.config.model
                ))
                .header("x-goog-api-key", &self.config.api_key)
                .json(&gemini_body(system, user))
        };
        // `without_url`: a reqwest error prints the URL, never the body; dropped anyway.
        let response = request
            .send()
            .await
            .map_err(|e| self.unavailable(e.without_url()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(self.unavailable(format!("status {status}")));
        }
        let reply: Value = response
            .json()
            .await
            .map_err(|_| self.unavailable("unreadable reply"))?;
        let read = if self.config.name == ANTHROPIC {
            read_anthropic(&reply)
        } else {
            read_gemini(&reply)
        };
        let (text, input_tokens, output_tokens) =
            read.ok_or_else(|| self.unavailable("reply without an answer"))?;
        Ok(ProviderReply {
            text,
            input_tokens,
            output_tokens,
        })
    }
}
