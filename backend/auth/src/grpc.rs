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
    req.extensions()
        .get::<AuthIdentity>()
        .map(|i| i.user_id.clone())
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
}
