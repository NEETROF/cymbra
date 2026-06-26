//! Internal-token tonic interceptor (task 2.5).
//!
//! Validates the backend's own access token on protected methods and injects
//! [`AuthIdentity`] into the request extensions. Provider tokens are never seen
//! here — only Cymbra ID's own tokens.

use crate::identity::AuthIdentity;
use crate::token;
use jsonwebtoken::DecodingKey;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::service::Interceptor;
use tonic::{Request, Status};

/// Verifies internal access tokens against the published signing keys and the
/// audience allow-list, then attaches the resolved identity.
#[derive(Clone)]
pub struct AuthInterceptor {
    keys: Arc<HashMap<String, DecodingKey>>,
    audiences: Arc<Vec<String>>,
}

impl AuthInterceptor {
    pub fn new(keys: HashMap<String, DecodingKey>, audiences: Vec<String>) -> Self {
        Self {
            keys: Arc::new(keys),
            audiences: Arc::new(audiences),
        }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        // Copy the bearer token out before mutating extensions (ends the borrow).
        let token_string = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string())
            .ok_or_else(|| Status::unauthenticated("missing bearer token"))?;

        let auds: Vec<&str> = self.audiences.iter().map(|s| s.as_str()).collect();
        let claims = token::verify(&token_string, &self.keys, &auds).map_err(|e| e.to_status())?;

        req.extensions_mut().insert(AuthIdentity {
            user_id: claims.sub,
            audience: claims.aud,
            roles: claims.roles,
        });
        Ok(req)
    }
}

/// Like [`AuthInterceptor`] but **does not reject** unauthenticated requests — it
/// injects [`AuthIdentity`] only when a valid token is present. Used on the auth
/// service, whose sign-in methods are public while link/unlink need the caller.
#[derive(Clone)]
pub struct OptionalAuthInterceptor {
    keys: Arc<HashMap<String, DecodingKey>>,
    audiences: Arc<Vec<String>>,
}

impl OptionalAuthInterceptor {
    pub fn new(keys: HashMap<String, DecodingKey>, audiences: Vec<String>) -> Self {
        Self {
            keys: Arc::new(keys),
            audiences: Arc::new(audiences),
        }
    }
}

impl Interceptor for OptionalAuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token_string = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string());
        if let Some(t) = token_string {
            let auds: Vec<&str> = self.audiences.iter().map(|s| s.as_str()).collect();
            if let Ok(claims) = token::verify(&t, &self.keys, &auds) {
                req.extensions_mut().insert(AuthIdentity {
                    user_id: claims.sub,
                    audience: claims.aud,
                    roles: claims.roles,
                });
            }
        }
        Ok(req)
    }
}
