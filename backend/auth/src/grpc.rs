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
        self.port
            .sign_up_local(&r.email, &r.password, &r.locale)
            .await?;
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
        let r = req.into_inner();
        self.port.resend_verification(&r.email, &r.locale).await?;
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
        let r = req.into_inner();
        self.port
            .request_password_reset(&r.email, &r.locale)
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

    async fn set_local_credential(
        &self,
        req: Request<proto::SetLocalCredentialRequest>,
    ) -> Result<Response<proto::SetLocalCredentialResponse>, Status> {
        let user_id = caller(&req)?;
        let r = req.into_inner();
        self.port
            .set_local_credential(&user_id, &r.email, &r.password, &r.locale)
            .await?;
        Ok(Response::new(proto::SetLocalCredentialResponse {}))
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
        // Admin-gated, and scoped by what the admin ADMINISTERS — not by the audience
        // of their own token. That was the previous rule, and since the console always
        // signs in as `back-office`, it cut the target's console sessions and left
        // their app sessions alive while reporting a successful revocation.
        let (admin, scope) = {
            let id = identity(&req)?;
            cymbra_platform::guard::require_admin(id)?;
            let scope = if id.has_role_in_scope("global", "admin") {
                // Break-glass: a platform admin cuts every session the account has.
                cymbra_auth_port::RevocationScope::All
            } else {
                // Otherwise: the app audiences this admin is entitled to, plus the
                // web + console surfaces, which are the identity product rather than
                // any one app. An empty list here would cut nothing, never everything.
                let mut auds = id.admin_scopes(&cymbra_platform::APP_SCOPES);
                auds.push(cymbra_platform::BACKOFFICE_AUDIENCE.to_string());
                auds.push("web".to_string());
                cymbra_auth_port::RevocationScope::Only(auds)
            };
            (id.user_id.clone(), scope)
        };
        let target = req.into_inner().user_id;
        self.port
            .revoke_account_sessions(&admin, &target, &scope)
            .await?;
        Ok(Response::new(proto::RevokeAccountSessionsResponse {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_auth_port::MockAuthPort;
    use mockall::predicate::eq;

    /// Wraps a configured mock port in the gRPC adapter under test.
    fn grpc(port: MockAuthPort) -> AuthGrpc<MockAuthPort> {
        AuthGrpc::new(Arc::new(port))
    }

    /// A request carrying a verified caller identity (as the interceptor would stamp).
    fn req_as<T>(body: T, user_id: &str, roles: &[&str]) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            ..Default::default()
        });
        req
    }

    #[tokio::test]
    async fn missing_identity_is_unauthenticated() {
        // No expectations: the port must never be reached when the caller is unverified.
        let g = grpc(MockAuthPort::new());
        // No AuthIdentity extension → the interceptor rejected/omitted it.
        let err = g
            .revoke_all_sessions(Request::new(proto::RevokeAllSessionsRequest {}))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn sign_out_everywhere_uses_the_caller_id() {
        let mut port = MockAuthPort::new();
        // The adapter must scope the revocation to the *caller's* id.
        port.expect_revoke_all_sessions()
            .with(eq("u1"))
            .times(1)
            .returning(|_| Ok(()));
        let g = grpc(port);
        g.revoke_all_sessions(req_as(proto::RevokeAllSessionsRequest {}, "u1", &["user"]))
            .await
            .unwrap();
        // `.with` + `.times(1)` are verified on drop.
    }

    /// An identity holding `admin` in one app scope, signed in to the console.
    fn req_scoped_admin<T>(body: T, user_id: &str, scope: &str) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: cymbra_platform::BACKOFFICE_AUDIENCE.into(),
            roles: vec!["user".into(), "admin".into()],
            roles_by_scope: [(scope.to_string(), vec!["admin".to_string()])]
                .into_iter()
                .collect(),
        });
        req
    }

    /// The revocation reaches the app the admin administers — NOT the audience their
    /// own token happens to carry. Before this rule, a console admin (always
    /// `back-office`) cut the target's console sessions and left the app ones alive.
    #[tokio::test]
    async fn revocation_is_scoped_by_what_the_admin_administers() {
        let mut port = MockAuthPort::new();
        port.expect_revoke_account_sessions()
            .withf(|admin, target, scope| {
                admin == "admin-1"
                    && target == "target"
                    && match scope {
                        cymbra_auth_port::RevocationScope::Only(auds) => {
                            auds.contains(&"music".to_string())
                                && auds.contains(&"back-office".to_string())
                                && !auds.contains(&"live".to_string())
                        }
                        cymbra_auth_port::RevocationScope::All => false,
                    }
            })
            .times(1)
            .returning(|_, _, _| Ok(()));
        let g = grpc(port);
        g.revoke_account_sessions(req_scoped_admin(
            proto::RevokeAccountSessionsRequest {
                user_id: "target".into(),
            },
            "admin-1",
            "music",
        ))
        .await
        .unwrap();
    }

    /// The `global` break-glass cuts every session the account has.
    #[tokio::test]
    async fn global_admin_revokes_every_audience() {
        let mut port = MockAuthPort::new();
        port.expect_revoke_account_sessions()
            .withf(|_, _, scope| matches!(scope, cymbra_auth_port::RevocationScope::All))
            .times(1)
            .returning(|_, _, _| Ok(()));
        let g = grpc(port);
        g.revoke_account_sessions(req_scoped_admin(
            proto::RevokeAccountSessionsRequest {
                user_id: "target".into(),
            },
            "admin-1",
            "global",
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn non_admin_cannot_revoke_a_target_account() {
        let mut port = MockAuthPort::new();
        // The admin gate must reject before the port is ever touched.
        port.expect_revoke_account_sessions().never();
        let g = grpc(port);
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
    }
}
