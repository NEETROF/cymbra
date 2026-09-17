// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Tonic adapter for LinguaDataService — thin transport (coverage-excluded); the logic is
//! host-tested in `data`. The caller is the token's user (from the interceptor), never
//! the request body.

#![allow(clippy::result_large_err)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::data::DataModule;
use crate::grpc_util::{caller, now_ms};
use crate::proto::lingua_data_service_server::LinguaDataService;
use crate::proto::{
    EraseMyDataRequest, EraseMyDataResponse, GetDataStateRequest, GetDataStateResponse,
};

pub struct DataGrpc {
    module: Arc<DataModule>,
}

impl DataGrpc {
    pub fn new(module: Arc<DataModule>) -> Self {
        Self { module }
    }
}

#[tonic::async_trait]
impl LinguaDataService for DataGrpc {
    async fn erase_my_data(
        &self,
        req: Request<EraseMyDataRequest>,
    ) -> Result<Response<EraseMyDataResponse>, Status> {
        let user = caller(&req)?;
        let erased_at = self.module.erase_my_data(&user, now_ms()).await?;
        Ok(Response::new(EraseMyDataResponse { erased_at }))
    }

    async fn get_data_state(
        &self,
        req: Request<GetDataStateRequest>,
    ) -> Result<Response<GetDataStateResponse>, Status> {
        let user = caller(&req)?;
        let erased_at = self.module.data_state(&user).await?;
        Ok(Response::new(GetDataStateResponse { erased_at }))
    }
}
