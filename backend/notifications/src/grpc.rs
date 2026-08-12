//! The push platform's gRPC **server** adapter (change: add-push-notifications,
//! task 1.3): exposes `NotificationService` by translating each RPC into a
//! [`PushRegistry`] call. The account written is always the authenticated
//! caller's — resolved from the internal-token interceptor's request extension,
//! never from the body.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::proto::{
    CategoryPref as ProtoCategoryPref, GetNotificationSettingsRequest,
    GetNotificationSettingsResponse, RegisterPushTokenRequest, RegisterPushTokenResponse,
    SetNotificationPrefRequest, SetNotificationPrefResponse, SetTimezoneRequest,
    SetTimezoneResponse, UnregisterPushTokenRequest, UnregisterPushTokenResponse,
    notification_service_server::{NotificationService, NotificationServiceServer},
};
use crate::repo::{Platform, PushRegistry};
use crate::select_core::validate_timezone;

/// Wraps the push registry as a tonic `NotificationService`.
pub struct NotificationGrpc<R: PushRegistry> {
    registry: Arc<R>,
}

impl<R: PushRegistry + 'static> NotificationGrpc<R> {
    pub fn new(registry: Arc<R>) -> Self {
        Self { registry }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> NotificationServiceServer<Self> {
        NotificationServiceServer::new(self)
    }
}

fn identity<T>(req: &Request<T>) -> Result<AuthIdentity, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

fn require_non_empty(value: &str, field: &str) -> Result<(), Status> {
    if value.trim().is_empty() {
        return Err(Status::invalid_argument(format!("{field} is required")));
    }
    Ok(())
}

#[tonic::async_trait]
impl<R: PushRegistry + 'static> NotificationService for NotificationGrpc<R> {
    async fn register_push_token(
        &self,
        req: Request<RegisterPushTokenRequest>,
    ) -> Result<Response<RegisterPushTokenResponse>, Status> {
        let id = identity(&req)?;
        let body = req.into_inner();
        require_non_empty(&body.token, "token")?;
        // Rejects windows/linux: they hold no deliverable token (design D7).
        let platform = Platform::parse(&body.platform).map_err(|e| e.to_status())?;
        self.registry
            .register_token(&id.user_id, body.token.trim(), platform)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(RegisterPushTokenResponse {}))
    }

    async fn unregister_push_token(
        &self,
        req: Request<UnregisterPushTokenRequest>,
    ) -> Result<Response<UnregisterPushTokenResponse>, Status> {
        let id = identity(&req)?;
        let body = req.into_inner();
        require_non_empty(&body.token, "token")?;
        self.registry
            .unregister_token(&id.user_id, body.token.trim())
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(UnregisterPushTokenResponse {}))
    }

    async fn set_notification_pref(
        &self,
        req: Request<SetNotificationPrefRequest>,
    ) -> Result<Response<SetNotificationPrefResponse>, Status> {
        let id = identity(&req)?;
        let body = req.into_inner();
        require_non_empty(&body.category, "category")?;
        self.registry
            .set_pref(&id.user_id, body.category.trim(), body.enabled)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(SetNotificationPrefResponse {}))
    }

    async fn set_timezone(
        &self,
        req: Request<SetTimezoneRequest>,
    ) -> Result<Response<SetTimezoneResponse>, Status> {
        let id = identity(&req)?;
        let tz = req.into_inner().timezone;
        // Empty is a no-op (mirrors SetLocale), so a client that cannot resolve a
        // timezone simply leaves the stored one alone.
        if tz.trim().is_empty() {
            return Ok(Response::new(SetTimezoneResponse {}));
        }
        let tz = validate_timezone(tz.trim()).map_err(|e| e.to_status())?;
        self.registry
            .set_timezone(&id.user_id, &tz)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(SetTimezoneResponse {}))
    }

    async fn get_notification_settings(
        &self,
        req: Request<GetNotificationSettingsRequest>,
    ) -> Result<Response<GetNotificationSettingsResponse>, Status> {
        let id = identity(&req)?;
        let prefs = self
            .registry
            .prefs(&id.user_id)
            .await
            .map_err(|e| e.to_status())?;
        let timezone = self
            .registry
            .timezone(&id.user_id)
            .await
            .map_err(|e| e.to_status())?;
        let has_registered_device = self
            .registry
            .has_device(&id.user_id)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(GetNotificationSettingsResponse {
            prefs: prefs
                .into_iter()
                .map(|p| ProtoCategoryPref {
                    category: p.category,
                    enabled: p.enabled,
                })
                .collect(),
            timezone,
            has_registered_device,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::{CategoryPref, MockPushRegistry};

    fn authed<T>(body: T, user_id: &str) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.to_string(),
            audience: "music".into(),
            roles: vec!["user".into()],
            roles_by_scope: Default::default(),
        });
        req
    }

    #[tokio::test]
    async fn register_writes_the_caller_account_not_the_body() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_register_token()
            .withf(|uid, token, platform| {
                uid == "caller" && token == "tok-1" && *platform == Platform::Ios
            })
            .times(1)
            .returning(|_, _, _| Ok(()));
        let g = NotificationGrpc::new(Arc::new(registry));
        g.register_push_token(authed(
            RegisterPushTokenRequest {
                token: " tok-1 ".into(),
                platform: "iOS".into(),
            },
            "caller",
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn register_rejects_desktop_platforms_and_empty_tokens() {
        let g = NotificationGrpc::new(Arc::new(MockPushRegistry::new()));
        let err = g
            .register_push_token(authed(
                RegisterPushTokenRequest {
                    token: "tok".into(),
                    platform: "windows".into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);

        let err = g
            .register_push_token(authed(
                RegisterPushTokenRequest {
                    token: "  ".into(),
                    platform: "ios".into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn unauthenticated_calls_are_refused() {
        let g = NotificationGrpc::new(Arc::new(MockPushRegistry::new()));
        let err = g
            .register_push_token(Request::new(RegisterPushTokenRequest {
                token: "tok".into(),
                platform: "ios".into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn unregister_is_owner_scoped() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_unregister_token()
            .withf(|uid, token| uid == "owner" && token == "tok-1")
            .times(1)
            .returning(|_, _| Ok(()));
        let g = NotificationGrpc::new(Arc::new(registry));
        g.unregister_push_token(authed(
            UnregisterPushTokenRequest {
                token: "tok-1".into(),
            },
            "owner",
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn set_pref_requires_a_category() {
        let g = NotificationGrpc::new(Arc::new(MockPushRegistry::new()));
        let err = g
            .set_notification_pref(authed(
                SetNotificationPrefRequest {
                    category: "".into(),
                    enabled: true,
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn set_pref_records_the_choice() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_set_pref()
            .withf(|uid, cat, enabled| uid == "u1" && cat == "practice_streak" && !*enabled)
            .times(1)
            .returning(|_, _, _| Ok(()));
        let g = NotificationGrpc::new(Arc::new(registry));
        g.set_notification_pref(authed(
            SetNotificationPrefRequest {
                category: "practice_streak".into(),
                enabled: false,
            },
            "u1",
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn set_timezone_stores_a_valid_zone_and_ignores_empty() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_set_timezone()
            .withf(|uid, tz| uid == "u1" && tz == "Europe/Paris")
            .times(1)
            .returning(|_, _| Ok(()));
        let g = NotificationGrpc::new(Arc::new(registry));
        g.set_timezone(authed(
            SetTimezoneRequest {
                timezone: " Europe/Paris ".into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        // Empty is a no-op: the mock would panic on a second call.
        g.set_timezone(authed(
            SetTimezoneRequest {
                timezone: "".into(),
            },
            "u1",
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn set_timezone_rejects_an_unknown_zone() {
        let g = NotificationGrpc::new(Arc::new(MockPushRegistry::new()));
        let err = g
            .set_timezone(authed(
                SetTimezoneRequest {
                    timezone: "Mars/Olympus".into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn settings_reports_prefs_timezone_and_device_presence() {
        let mut registry = MockPushRegistry::new();
        registry.expect_prefs().returning(|_| {
            Ok(vec![CategoryPref {
                category: "practice_streak".into(),
                enabled: true,
            }])
        });
        registry
            .expect_timezone()
            .returning(|_| Ok(Some("Europe/Paris".into())));
        registry.expect_has_device().returning(|_| Ok(true));
        let g = NotificationGrpc::new(Arc::new(registry));
        let res = g
            .get_notification_settings(authed(GetNotificationSettingsRequest {}, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(res.prefs.len(), 1);
        assert_eq!(res.prefs[0].category, "practice_streak");
        assert!(res.prefs[0].enabled);
        assert_eq!(res.timezone.as_deref(), Some("Europe/Paris"));
        assert!(res.has_registered_device);
    }
}
