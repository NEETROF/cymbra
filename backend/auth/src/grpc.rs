//! The auth module's gRPC **server** adapter (task 4.10): exposes `AuthService`
//! by translating each RPC into an [`AuthPort`] call.
//!
//! Sign-up / verification / sign-in / refresh / logout / reset are **public**.
//! `LinkIdentity` / `UnlinkIdentity` are **authenticated**: the caller's
//! `user_id` comes from the internal-token interceptor (request extension). The
//! server (group 5) mounts the public methods without the interceptor.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_auth_port::proto::{
    self,
    auth_service_server::{AuthService, AuthServiceServer},
};
use cymbra_auth_port::{AuthPort, TokenPair};
use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

pub struct AuthGrpc<P: AuthPort> {
    port: Arc<P>,
}

impl<P: AuthPort + 'static> AuthGrpc<P> {
    pub fn new(port: Arc<P>) -> Self {
        Self { port }
    }

    pub fn into_server(self) -> AuthServiceServer<Self> {
        AuthServiceServer::new(self)
    }
}

fn token_pair(p: TokenPair) -> proto::TokenPair {
    proto::TokenPair {
        access_token: p.access_token,
        refresh_token: p.refresh_token,
    }
}

fn caller<T>(req: &Request<T>) -> Result<String, Status> {
    identity(req).map(|i| i.user_id.clone())
}

/// The verified caller identity (user_id + effective roles), stamped by the
/// internal-token interceptor. Absent when no valid access token was presented.
fn identity<T>(req: &Request<T>) -> Result<&AuthIdentity, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

#[tonic::async_trait]
impl<P: AuthPort + 'static> AuthService for AuthGrpc<P> {
    async fn sign_up_local(
        &self,
        req: Request<proto::SignUpLocalRequest>,
    ) -> Result<Response<proto::SignUpLocalResponse>, Status> {
        let r = req.into_inner();
        self.port.sign_up_local(&r.email, &r.password).await?;
        Ok(Response::new(proto::SignUpLocalResponse {}))
    }

    async fn verify_email(
        &self,
        req: Request<proto::VerifyEmailRequest>,
    ) -> Result<Response<proto::VerifyEmailResponse>, Status> {
        self.port.verify_email(&req.into_inner().token).await?;
        Ok(Response::new(proto::VerifyEmailResponse {}))
    }

    async fn resend_verification(
        &self,
        req: Request<proto::ResendVerificationRequest>,
    ) -> Result<Response<proto::ResendVerificationResponse>, Status> {
        self.port
            .resend_verification(&req.into_inner().email)
            .await?;
        Ok(Response::new(proto::ResendVerificationResponse {}))
    }

    async fn sign_in_local(
        &self,
        req: Request<proto::SignInLocalRequest>,
    ) -> Result<Response<proto::TokenPair>, Status> {
        let r = req.into_inner();
        let pair = self
            .port
            .sign_in_local(&r.email, &r.password, &r.audience)
            .await?;
        Ok(Response::new(token_pair(pair)))
    }

    async fn sign_in_oidc(
        &self,
        req: Request<proto::SignInOidcRequest>,
    ) -> Result<Response<proto::TokenPair>, Status> {
        let r = req.into_inner();
        let pair = self.port.sign_in_oidc(&r.id_token, &r.audience).await?;
        Ok(Response::new(token_pair(pair)))
    }

    async fn refresh(
        &self,
        req: Request<proto::RefreshRequest>,
    ) -> Result<Response<proto::TokenPair>, Status> {
        let pair = self.port.refresh(&req.into_inner().refresh_token).await?;
        Ok(Response::new(token_pair(pair)))
    }

    async fn logout(
        &self,
        req: Request<proto::LogoutRequest>,
    ) -> Result<Response<proto::LogoutResponse>, Status> {
        self.port.logout(&req.into_inner().refresh_token).await?;
        Ok(Response::new(proto::LogoutResponse {}))
    }

    async fn request_password_reset(
        &self,
        req: Request<proto::RequestPasswordResetRequest>,
    ) -> Result<Response<proto::RequestPasswordResetResponse>, Status> {
        self.port
            .request_password_reset(&req.into_inner().email)
            .await?;
        Ok(Response::new(proto::RequestPasswordResetResponse {}))
    }

    async fn reset_password(
        &self,
        req: Request<proto::ResetPasswordRequest>,
    ) -> Result<Response<proto::ResetPasswordResponse>, Status> {
        let r = req.into_inner();
        self.port.reset_password(&r.token, &r.new_password).await?;
        Ok(Response::new(proto::ResetPasswordResponse {}))
    }

    async fn link_identity(
        &self,
        req: Request<proto::LinkIdentityRequest>,
    ) -> Result<Response<proto::LinkIdentityResponse>, Status> {
        let user_id = caller(&req)?;
        self.port
            .link_identity(&user_id, &req.into_inner().id_token)
            .await?;
        Ok(Response::new(proto::LinkIdentityResponse {}))
    }

    async fn unlink_identity(
        &self,
        req: Request<proto::UnlinkIdentityRequest>,
    ) -> Result<Response<proto::UnlinkIdentityResponse>, Status> {
        let user_id = caller(&req)?;
        let r = req.into_inner();
        self.port
            .unlink_identity(&user_id, &r.provider, &r.subject)
            .await?;
        Ok(Response::new(proto::UnlinkIdentityResponse {}))
    }

    async fn revoke_all_sessions(
        &self,
        req: Request<proto::RevokeAllSessionsRequest>,
    ) -> Result<Response<proto::RevokeAllSessionsResponse>, Status> {
        let user_id = caller(&req)?;
        self.port.revoke_all_sessions(&user_id).await?;
        Ok(Response::new(proto::RevokeAllSessionsResponse {}))
    }

    async fn revoke_account_sessions(
        &self,
        req: Request<proto::RevokeAccountSessionsRequest>,
    ) -> Result<Response<proto::RevokeAccountSessionsResponse>, Status> {
        // Admin-gated: only an admin may cut off another account's sessions, and only
        // within the audience their admin role is scoped to (the token's audience).
        let (admin, audience) = {
            let id = identity(&req)?;
            cymbra_platform::guard::require_admin(id)?;
            (id.user_id.clone(), id.audience.clone())
        };
        let target = req.into_inner().user_id;
        self.port
            .revoke_account_sessions(&admin, &target, &audience)
            .await?;
        Ok(Response::new(proto::RevokeAccountSessionsResponse {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_platform::Result;
    use std::sync::Mutex;

    /// Records which port method the adapter routed to (with args), so the tests can
    /// assert the caller scoping + admin gate without any real session store.
    #[derive(Default)]
    struct Calls {
        revoke_all: Mutex<Vec<String>>,
        admin_revoke: Mutex<Vec<(String, String, String)>>,
    }

    struct FakeAuth {
        calls: Arc<Calls>,
    }

    #[async_trait::async_trait]
    impl AuthPort for FakeAuth {
        async fn revoke_all_sessions(&self, user_id: &str) -> Result<()> {
            self.calls.revoke_all.lock().unwrap().push(user_id.into());
            Ok(())
        }
        async fn revoke_account_sessions(
            &self,
            admin: &str,
            target: &str,
            audience: &str,
        ) -> Result<()> {
            self.calls.admin_revoke.lock().unwrap().push((
                admin.into(),
                target.into(),
                audience.into(),
            ));
            Ok(())
        }
        // The session-management RPCs never touch the methods below.
        async fn sign_up_local(&self, _: &str, _: &str) -> Result<()> {
            unreachable!()
        }
        async fn verify_email(&self, _: &str) -> Result<()> {
            unreachable!()
        }
        async fn resend_verification(&self, _: &str) -> Result<()> {
            unreachable!()
        }
        async fn sign_in_local(&self, _: &str, _: &str, _: &str) -> Result<TokenPair> {
            unreachable!()
        }
        async fn sign_in_oidc(&self, _: &str, _: &str) -> Result<TokenPair> {
            unreachable!()
        }
        async fn refresh(&self, _: &str) -> Result<TokenPair> {
            unreachable!()
        }
        async fn logout(&self, _: &str) -> Result<()> {
            unreachable!()
        }
        async fn request_password_reset(&self, _: &str) -> Result<()> {
            unreachable!()
        }
        async fn reset_password(&self, _: &str, _: &str) -> Result<()> {
            unreachable!()
        }
        async fn link_identity(&self, _: &str, _: &str) -> Result<()> {
            unreachable!()
        }
        async fn unlink_identity(&self, _: &str, _: &str, _: &str) -> Result<()> {
            unreachable!()
        }
    }

    fn grpc() -> (AuthGrpc<FakeAuth>, Arc<Calls>) {
        let calls = Arc::new(Calls::default());
        let port = Arc::new(FakeAuth {
            calls: calls.clone(),
        });
        (AuthGrpc::new(port), calls)
    }

    /// A request carrying a verified caller identity (as the interceptor would stamp).
    fn req_as<T>(body: T, user_id: &str, roles: &[&str]) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
        });
        req
    }

    #[tokio::test]
    async fn missing_identity_is_unauthenticated() {
        let (g, _) = grpc();
        // No AuthIdentity extension → the interceptor rejected/omitted it.
        let err = g
            .revoke_all_sessions(Request::new(proto::RevokeAllSessionsRequest {}))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn sign_out_everywhere_uses_the_caller_id() {
        let (g, calls) = grpc();
        g.revoke_all_sessions(req_as(proto::RevokeAllSessionsRequest {}, "u1", &["user"]))
            .await
            .unwrap();
        assert_eq!(*calls.revoke_all.lock().unwrap(), vec!["u1".to_string()]);
    }

    #[tokio::test]
    async fn admin_can_revoke_a_target_account() {
        let (g, calls) = grpc();
        g.revoke_account_sessions(req_as(
            proto::RevokeAccountSessionsRequest {
                user_id: "target".into(),
            },
            "admin-1",
            &["user", "admin"],
        ))
        .await
        .unwrap();
        // The admin's own token audience ("music") scopes the revocation.
        assert_eq!(
            *calls.admin_revoke.lock().unwrap(),
            vec![(
                "admin-1".to_string(),
                "target".to_string(),
                "music".to_string()
            )]
        );
    }

    #[tokio::test]
    async fn non_admin_cannot_revoke_a_target_account() {
        let (g, calls) = grpc();
        let err = g
            .revoke_account_sessions(req_as(
                proto::RevokeAccountSessionsRequest {
                    user_id: "target".into(),
                },
                "mod-1",
                &["user", "moderator"],
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        assert!(calls.admin_revoke.lock().unwrap().is_empty());
    }
}
