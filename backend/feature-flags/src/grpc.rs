//! The gRPC `FlagService` adapter: the client read (`GetEffectiveFlags`, mountable
//! unauthenticated for pre-account UI) plus the admin surface (`ListFlagDefinitions`,
//! `SetFlag`/`SetConfig`/`ClearOverride`, `ListFlagChanges`). Thin translation over
//! [`FlagService`]; authorization + scoping live in the service. Mount behind the
//! `OptionalAuthInterceptor` so the read works with or without a token while the
//! admin methods still require an authenticated admin.
#![allow(clippy::result_large_err)]

use crate::proto::flag_service_server::{FlagService as FlagServiceTrait, FlagServiceServer};
use crate::proto::{self};
use crate::service::{Actor, FlagService, KeyState};
use crate::store::ChangeRecord;
use crate::value::FlagValue;
use crate::{EvalContext, RolloutScope};
use cymbra_platform::error::{AppError, Result};
use cymbra_platform::identity::AuthIdentity;
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// gRPC adapter wrapping the shared [`FlagService`].
pub struct FlagGrpc {
    svc: Arc<FlagService>,
}

impl FlagGrpc {
    pub fn new(svc: Arc<FlagService>) -> Self {
        Self { svc }
    }

    /// Package the adapter as a tonic server (caller adds the interceptor).
    pub fn into_server(self) -> FlagServiceServer<Self> {
        FlagServiceServer::new(self)
    }

    /// Extract an authenticated admin actor, or a gRPC error. Used by every admin
    /// method; the read method uses the optional identity directly.
    fn admin_actor<T>(&self, req: &Request<T>) -> std::result::Result<Actor, Status> {
        let id = req
            .extensions()
            .get::<AuthIdentity>()
            .cloned()
            .ok_or_else(|| Status::unauthenticated("missing identity"))?;
        cymbra_platform::guard::require_admin(&id)?;
        Ok(Actor {
            user_id: id.user_id,
            app: id.audience,
            is_admin: true,
        })
    }
}

#[tonic::async_trait]
impl FlagServiceTrait for FlagGrpc {
    async fn get_effective_flags(
        &self,
        req: Request<proto::GetEffectiveFlagsRequest>,
    ) -> std::result::Result<Response<proto::GetEffectiveFlagsResponse>, Status> {
        let identity = req.extensions().get::<AuthIdentity>().cloned();
        let r = req.into_inner();
        // Authenticated ⇒ the token audience + roles (the `app` field is ignored so
        // it can't widen the set); unauthenticated ⇒ the anonymous set for the app
        // the caller named.
        let ctx = match &identity {
            Some(id) => EvalContext::authenticated(&id.audience, &id.roles),
            None => EvalContext::anonymous(&r.app),
        };
        let set = self.svc.effective(&ctx);
        if !r.known_version.is_empty() && r.known_version == set.version {
            return Ok(Response::new(proto::GetEffectiveFlagsResponse {
                version: set.version,
                unchanged: true,
                flags: vec![],
            }));
        }
        let flags = set
            .entries
            .iter()
            .map(|e| proto::EffectiveFlag {
                key: e.key.clone(),
                value: Some(value_to_proto(&e.value)),
            })
            .collect();
        Ok(Response::new(proto::GetEffectiveFlagsResponse {
            version: set.version,
            unchanged: false,
            flags,
        }))
    }

    async fn list_flag_definitions(
        &self,
        req: Request<proto::ListFlagDefinitionsRequest>,
    ) -> std::result::Result<Response<proto::ListFlagDefinitionsResponse>, Status> {
        let actor = self.admin_actor(&req)?;
        let r = req.into_inner();
        let filter = (!r.app_filter.is_empty()).then_some(r.app_filter.as_str());
        let definitions = self
            .svc
            .definitions_for(&actor, filter)
            .await?
            .iter()
            .map(|(ks, editable)| definition_to_proto(ks, *editable))
            .collect();
        Ok(Response::new(proto::ListFlagDefinitionsResponse {
            definitions,
        }))
    }

    async fn set_flag(
        &self,
        req: Request<proto::SetFlagRequest>,
    ) -> std::result::Result<Response<proto::SetFlagResponse>, Status> {
        let actor = self.admin_actor(&req)?;
        let r = req.into_inner();
        let rollout = parse_rollout_opt(&r.rollout_scope)?;
        let ks = self
            .svc
            .set_value(
                &actor,
                &r.app,
                &r.key,
                FlagValue::Bool(r.enabled),
                rollout,
                r.confirm,
            )
            .await?;
        Ok(Response::new(proto::SetFlagResponse {
            definition: Some(definition_to_proto(&ks, true)),
        }))
    }

    async fn set_config(
        &self,
        req: Request<proto::SetConfigRequest>,
    ) -> std::result::Result<Response<proto::SetConfigResponse>, Status> {
        let actor = self.admin_actor(&req)?;
        let r = req.into_inner();
        let value = value_from_proto(r.value.as_ref())?;
        let rollout = parse_rollout_opt(&r.rollout_scope)?;
        let ks = self
            .svc
            .set_value(&actor, &r.app, &r.key, value, rollout, r.confirm)
            .await?;
        Ok(Response::new(proto::SetConfigResponse {
            definition: Some(definition_to_proto(&ks, true)),
        }))
    }

    async fn clear_override(
        &self,
        req: Request<proto::ClearOverrideRequest>,
    ) -> std::result::Result<Response<proto::ClearOverrideResponse>, Status> {
        let actor = self.admin_actor(&req)?;
        let r = req.into_inner();
        let ks = self
            .svc
            .clear_value(&actor, &r.app, &r.key, r.confirm)
            .await?;
        Ok(Response::new(proto::ClearOverrideResponse {
            definition: Some(definition_to_proto(&ks, true)),
        }))
    }

    async fn list_flag_changes(
        &self,
        req: Request<proto::ListFlagChangesRequest>,
    ) -> std::result::Result<Response<proto::ListFlagChangesResponse>, Status> {
        let _ = self.admin_actor(&req)?;
        let r = req.into_inner();
        let app = (!r.app_filter.is_empty()).then_some(r.app_filter.as_str());
        let key = (!r.key.is_empty()).then_some(r.key.as_str());
        let limit = if r.limit == 0 { 100 } else { r.limit as i64 };
        let changes = self.svc.recent_changes(app, key, limit).await?;
        Ok(Response::new(proto::ListFlagChangesResponse {
            changes: changes.iter().map(change_to_proto).collect(),
        }))
    }
}

// --- pure conversion helpers (unit-tested) ---------------------------------

fn value_to_proto(v: &FlagValue) -> proto::FlagValue {
    use proto::flag_value::Kind;
    let kind = match v {
        FlagValue::Bool(b) => Kind::BoolValue(*b),
        FlagValue::Int(i) => Kind::IntValue(*i),
        FlagValue::Number(n) => Kind::NumberValue(*n),
        FlagValue::String(s) => Kind::StringValue(s.clone()),
        FlagValue::Json(j) => Kind::JsonValue(j.to_string()),
    };
    proto::FlagValue { kind: Some(kind) }
}

fn value_from_proto(v: Option<&proto::FlagValue>) -> Result<FlagValue> {
    use proto::flag_value::Kind;
    let kind = v
        .and_then(|v| v.kind.as_ref())
        .ok_or_else(|| AppError::InvalidArgument("missing value".into()))?;
    Ok(match kind {
        Kind::BoolValue(b) => FlagValue::Bool(*b),
        Kind::IntValue(i) => FlagValue::Int(*i),
        Kind::NumberValue(n) => FlagValue::Number(*n),
        Kind::StringValue(s) => FlagValue::String(s.clone()),
        Kind::JsonValue(s) => {
            let j: serde_json::Value = serde_json::from_str(s)
                .map_err(|e| AppError::InvalidArgument(format!("invalid json: {e}")))?;
            if !(j.is_object() || j.is_array()) {
                return Err(AppError::InvalidArgument(
                    "json value must be an object or array".into(),
                ));
            }
            FlagValue::Json(j)
        }
    })
}

fn definition_to_proto(k: &KeyState, editable: bool) -> proto::FlagDefinition {
    proto::FlagDefinition {
        key: k.def.key.to_string(),
        app: k.def.app.to_string(),
        value_type: k.def.value_type.as_str().to_string(),
        default_value: Some(value_to_proto(&k.def.default)),
        effective_value: Some(value_to_proto(&k.effective)),
        has_override: k.has_override,
        rollout_scope: k.rollout.as_str().to_string(),
        sensitive: k.def.sensitive,
        doc: k.def.doc.to_string(),
        editable,
        updated_by: k.updated_by.clone().unwrap_or_default(),
        updated_at: k.updated_at.clone().unwrap_or_default(),
    }
}

fn change_to_proto(c: &ChangeRecord) -> proto::FlagChange {
    proto::FlagChange {
        key: c.key.clone(),
        app: c.app.clone(),
        old_value: c.old_value.clone().unwrap_or_default(),
        new_value: c.new_value.clone(),
        actor: c.actor.clone(),
        // Handle resolution needs the user-account schema, which the isolated
        // flags role cannot read; the BO shows the actor id.
        actor_handle: String::new(),
        at: c.at.to_rfc3339(),
    }
}

fn parse_rollout_opt(s: &str) -> std::result::Result<Option<RolloutScope>, Status> {
    if s.is_empty() {
        return Ok(None);
    }
    RolloutScope::parse(s)
        .map(Some)
        .ok_or_else(|| Status::invalid_argument(format!("unknown rollout scope `{s}`")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ValueType;
    use serde_json::json;

    #[test]
    fn flag_value_proto_round_trips() {
        let cases = [
            FlagValue::Bool(true),
            FlagValue::Int(-3),
            FlagValue::Number(2.5),
            FlagValue::String("x".into()),
            FlagValue::Json(json!({"a": [1, 2]})),
        ];
        for v in cases {
            let back = value_from_proto(Some(&value_to_proto(&v))).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn value_from_proto_rejects_missing_and_scalar_json() {
        assert!(value_from_proto(None).is_err());
        assert!(value_from_proto(Some(&proto::FlagValue { kind: None })).is_err());
        // a JSON scalar is not a valid json-typed value
        let scalar = proto::FlagValue {
            kind: Some(proto::flag_value::Kind::JsonValue("3".into())),
        };
        assert!(value_from_proto(Some(&scalar)).is_err());
    }

    #[test]
    fn rollout_opt_parses() {
        assert_eq!(parse_rollout_opt("").unwrap(), None);
        assert_eq!(
            parse_rollout_opt("global").unwrap(),
            Some(RolloutScope::Global)
        );
        assert_eq!(
            parse_rollout_opt("staff_only").unwrap(),
            Some(RolloutScope::StaffOnly)
        );
        assert!(parse_rollout_opt("cohort").is_err());
    }

    use crate::invalidation::NoopBus;
    use crate::registry::{self, APP_MUSIC, Registry};
    use crate::resolver::MockAdminScopeResolver;
    use crate::store::MockFlagStore;
    use cymbra_platform::identity::AuthIdentity;

    fn grpc(platform_admin: bool) -> FlagGrpc {
        let mut store = MockFlagStore::new();
        store.expect_load_all().returning(|| Ok(vec![]));
        store.expect_upsert().returning(|_| Ok(()));
        let mut resolver = MockAdminScopeResolver::new();
        resolver
            .expect_is_platform_admin()
            .returning(move |_| Ok(platform_admin));
        let svc = Arc::new(FlagService::new(
            Registry::default(),
            Some(Arc::new(store)),
            Arc::new(NoopBus),
            Arc::new(resolver),
        ));
        FlagGrpc::new(svc)
    }

    fn admin_req<T>(msg: T, audience: &str) -> Request<T> {
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: "00000000-0000-0000-0000-0000000000aa".into(),
            audience: audience.into(),
            roles: vec!["admin".into()],
            ..Default::default()
        });
        req
    }

    #[tokio::test]
    async fn get_effective_unauthenticated_returns_anonymous_set_and_etag() {
        let g = grpc(false);
        let resp = g
            .get_effective_flags(Request::new(proto::GetEffectiveFlagsRequest {
                known_version: String::new(),
                app: APP_MUSIC.into(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(!resp.unchanged);
        assert!(!resp.flags.is_empty());
        // a second call with the returned version answers "unchanged" cheaply
        let again = g
            .get_effective_flags(Request::new(proto::GetEffectiveFlagsRequest {
                known_version: resp.version.clone(),
                app: APP_MUSIC.into(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(again.unchanged);
        assert!(again.flags.is_empty());
    }

    #[tokio::test]
    async fn admin_methods_reject_missing_identity() {
        let g = grpc(false);
        let err = g
            .list_flag_definitions(Request::new(proto::ListFlagDefinitionsRequest {
                app_filter: String::new(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn set_flag_handler_writes_for_admin() {
        let g = grpc(false);
        let resp = g
            .set_flag(admin_req(
                proto::SetFlagRequest {
                    key: registry::REWARDS_ENABLED.into(),
                    app: APP_MUSIC.into(),
                    enabled: true,
                    rollout_scope: "global".into(),
                    confirm: false,
                },
                APP_MUSIC,
            ))
            .await
            .unwrap()
            .into_inner();
        let def = resp.definition.unwrap();
        assert!(def.has_override);
        assert_eq!(
            def.effective_value.unwrap().kind,
            Some(proto::flag_value::Kind::BoolValue(true))
        );
    }

    #[tokio::test]
    async fn set_flag_handler_rejects_non_admin_scope_for_shared_key() {
        // per-app admin (resolver=false) cannot set an `all`-scoped key
        let g = grpc(false);
        let err = g
            .set_flag(admin_req(
                proto::SetFlagRequest {
                    key: registry::PLATFORM_MAINTENANCE.into(),
                    app: crate::context::APP_ALL.into(),
                    enabled: true,
                    rollout_scope: String::new(),
                    confirm: false,
                },
                APP_MUSIC,
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn list_definitions_handler_returns_declared_keys() {
        let g = grpc(true);
        let resp = g
            .list_flag_definitions(admin_req(
                proto::ListFlagDefinitionsRequest {
                    app_filter: String::new(),
                },
                APP_MUSIC,
            ))
            .await
            .unwrap()
            .into_inner();
        assert!(
            resp.definitions
                .iter()
                .any(|d| d.key == registry::RATING_ENABLED)
        );
    }

    #[test]
    fn definition_to_proto_carries_metadata() {
        let ks = KeyState {
            def: crate::registry::Registry::default()
                .get_by_key(crate::registry::RATING_REVIEW_MIN_VOTES)
                .unwrap()
                .clone(),
            effective: FlagValue::Int(9),
            has_override: true,
            rollout: RolloutScope::Global,
            updated_by: Some("00000000-0000-0000-0000-0000000000aa".into()),
            updated_at: Some("2026-07-31T00:00:00+00:00".into()),
        };
        let p = definition_to_proto(&ks, true);
        assert_eq!(p.key, crate::registry::RATING_REVIEW_MIN_VOTES);
        assert_eq!(p.value_type, ValueType::Int.as_str());
        assert!(p.has_override);
        assert_eq!(
            p.effective_value.unwrap().kind,
            Some(proto::flag_value::Kind::IntValue(9))
        );
    }
}
