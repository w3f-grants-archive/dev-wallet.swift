//===------------ substrate.rs --------------------------------------------===//
//  Copyright 2023, Tesseract Systems, Inc.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//===----------------------------------------------------------------------===//

use std::sync::Arc;

use async_trait::async_trait;

use tesseract::Result;
use tesseract_protocol_substrate::{AccountType, GetAccountResponse, Substrate};

use crate::request::{SubstrateAccount, SubstrateSign};
use crate::ui::{UI, UIProtocol};
use crate::settings::{SettingsProvider, TestSettingsProvider};

use super::parse::parse_transaction;
use super::wallet::Wallet;

pub(crate) struct SubstrateService {
    ui: Arc<UI>,
    settings_provider: Arc<SettingsProvider>
}

impl SubstrateService {
    pub fn new(ui: Arc<UI>, settings_provider: Arc<SettingsProvider>) -> Self {
        Self { ui: ui, settings_provider: settings_provider }
    }
}

const WALLET_PHRASE: &str =
    "arch flush fabric dentist fade service chronic bacon plunge expand still uncover";

#[async_trait]
impl tesseract_protocol_substrate::SubstrateService for SubstrateService {
    async fn get_account(self: Arc<Self>, account_type: AccountType) -> Result<GetAccountResponse> {
        let wallet = Wallet::new(WALLET_PHRASE).map_err(|e| e.into())?;
        let path = "".to_string();

        let key = wallet.derive(&path).map_err(|e| e.into())?;
        let strkey = key.to_string();

        let request = SubstrateAccount {
            algorithm: "Sr25519".to_owned(), //TODO: read from param
            path: path.clone(),
            key: strkey
        };

        let allow = self.ui.request_user_confirmation(request).await.map_err(|e| e.into())?;

        if allow {
            let veckey = key.to_vec();

            Ok(GetAccountResponse {
                public_key: veckey,
                path: path
            })
        } else {
            Err(tesseract::Error::kinded(tesseract::ErrorKind::Cancelled))
        }
    }

    async fn sign_transaction(
        self: Arc<Self>,
        account_type: AccountType,
        account_path: &str,
        extrinsic_data: &[u8],
        extrinsic_metadata: &[u8],
        extrinsic_types: &[u8],
    ) -> Result<Vec<u8>> {
        let data = parse_transaction(extrinsic_data, extrinsic_metadata, extrinsic_types).map_err(|e| e.into())?;

        let wallet = Wallet::new(WALLET_PHRASE).map_err(|e| e.into())?;

        let key = wallet.derive(account_path).map_err(|e| e.into())?;
        let strkey = key.to_string();

        let request = SubstrateSign {
            algorithm: "Sr25519".to_owned(), //TODO: read from param
            path: account_path.to_owned(),
            key: strkey,
            data: data
        };

        let allow = self.ui.request_user_confirmation(request).await.map_err(|e| e.into())?;

        if allow {
            wallet.sign(extrinsic_data).map_err(|e| e.into())
        } else {
            Err(tesseract::Error::kinded(tesseract::ErrorKind::Cancelled))
        }
    }
}

impl tesseract::service::Service for SubstrateService {
    type Protocol = Substrate;

    fn protocol(&self) -> &Substrate {
        &Substrate::Protocol
    }

    fn to_executor(self) -> Box<dyn tesseract::service::Executor + Send + Sync> {
        Box::new(tesseract_protocol_substrate::service::SubstrateExecutor::from_service(
            self,
        ))
    }
}