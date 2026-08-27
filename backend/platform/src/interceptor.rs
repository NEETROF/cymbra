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
            roles_by_scope: claims.roles_by_scope,
        });
        Ok(req)
    }
}

/// Like [`AuthInterceptor`] but **does not reject** requests that carry no token
/// at all — it injects [`AuthIdentity`] only when one is present. Used on the
/// auth service, whose sign-in methods are public while link/unlink need the
/// caller, and on the flag read, which resolves a pre-account UI's kill-switches.
///
/// "Optional" is about the **header**, never about its contents: a request that
/// presents a bearer is asserting an identity, so a token that fails
/// verification is rejected with `UNAUTHENTICATED` exactly as it would be on a
/// protected method. Swallowing it and answering as *anonymous* conflates
/// "nobody is signed in" with "your session expired" — two answers a caller
/// cannot tell apart, because the response carries no identity. That is what
/// took the drum beta's entry point away at random: an expired access token on
/// the flag poll came back as the anonymous set, complete with a legitimate
/// version, so the client stored it and lost `beta:midi-drums` until the next
/// poll happened to run after some *other* RPC had refreshed the token. Failing
/// here is what gives the client its cue to refresh and retry.
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
            // Presented and invalid ⇒ rejected, never downgraded to anonymous.
            let claims = token::verify(&t, &self.keys, &auds).map_err(|e| e.to_status())?;
            req.extensions_mut().insert(AuthIdentity {
                user_id: claims.sub,
                audience: claims.aud,
                roles: claims.roles,
                roles_by_scope: claims.roles_by_scope,
            });
        }
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Claims, encoding_key, new_claims, sign};
    use std::collections::BTreeMap;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // Throwaway Ed25519 keypair for tests only (same pair as `token`'s own).
    const PRIV: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIPlT7JHCc7NTTIZVmlCgVeNNEkqsENhAZoscpnG+jSSw\n-----END PRIVATE KEY-----\n";
    const PUB: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAiCcon5VNqPUMVYki6MnxJdscxMozrXbjmdiLGUL8sqA=\n-----END PUBLIC KEY-----\n";

    fn keys() -> HashMap<String, DecodingKey> {
        HashMap::from([("k1".to_string(), crate::token::decoding_key(PUB).unwrap())])
    }

    fn valid_token() -> String {
        let claims = new_claims("u1", "music", vec!["user".into()], Duration::from_secs(900));
        sign(&claims, "k1", &encoding_key(PRIV).unwrap()).unwrap()
    }

    fn expired_token() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let claims = Claims {
            sub: "u1".into(),
            aud: "music".into(),
            roles: vec!["user".into()],
            roles_by_scope: BTreeMap::new(),
            iat: now - 1000,
            exp: now - 600, // past, beyond the default leeway
            jti: "j".into(),
        };
        sign(&claims, "k1", &encoding_key(PRIV).unwrap()).unwrap()
    }

    fn request_with(bearer: Option<&str>) -> Request<()> {
        let mut req = Request::new(());
        if let Some(b) = bearer {
            req.metadata_mut()
                .insert("authorization", format!("Bearer {b}").parse().unwrap());
        }
        req
    }

    fn optional() -> OptionalAuthInterceptor {
        OptionalAuthInterceptor::new(keys(), vec!["music".to_string()])
    }

    #[test]
    fn no_header_at_all_stays_anonymous() {
        let req = optional().call(request_with(None)).expect("accepted");
        assert!(
            req.extensions().get::<AuthIdentity>().is_none(),
            "a caller that presented nothing asserted nothing"
        );
    }

    #[test]
    fn a_valid_bearer_is_resolved() {
        let token = valid_token();
        let req = optional()
            .call(request_with(Some(&token)))
            .expect("accepted");
        let id = req.extensions().get::<AuthIdentity>().expect("identity");
        assert_eq!(id.user_id, "u1");
        assert_eq!(id.audience, "music");
    }

    /// The regression the drum beta's vanishing entry point came down to: an
    /// expired token used to be swallowed, and the caller was answered as
    /// anonymous — indistinguishable, to the client, from being signed out.
    #[test]
    fn an_expired_bearer_is_rejected_not_downgraded_to_anonymous() {
        let token = expired_token();
        let status = optional()
            .call(request_with(Some(&token)))
            .expect_err("an asserted identity that does not verify is an error");
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn a_bearer_for_another_audience_is_rejected() {
        let token = valid_token();
        let mut interceptor = OptionalAuthInterceptor::new(keys(), vec!["live".to_string()]);
        let status = interceptor
            .call(request_with(Some(&token)))
            .expect_err("wrong audience");
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn a_garbage_bearer_is_rejected() {
        let status = optional()
            .call(request_with(Some("not-a-jwt")))
            .expect_err("malformed");
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn the_strict_interceptor_still_requires_a_bearer() {
        let mut strict = AuthInterceptor::new(keys(), vec!["music".to_string()]);
        let status = strict.call(request_with(None)).expect_err("missing bearer");
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }
}
