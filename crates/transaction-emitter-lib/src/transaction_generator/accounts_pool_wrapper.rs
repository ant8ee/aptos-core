// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::transaction_generator::{TransactionGenerator, TransactionGeneratorCreator};
use aptos_infallible::RwLock;
use aptos_logger::{sample, sample::SampleRate, warn};
use aptos_sdk::types::{transaction::SignedTransaction, LocalAccount};
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};

pub struct AccountsPoolWrapperGenerator {
    creator: Box<dyn TransactionGenerator>,
    accounts_pool: Arc<RwLock<Vec<LocalAccount>>>,
}

impl AccountsPoolWrapperGenerator {
    pub fn new(
        creator: Box<dyn TransactionGenerator>,
        accounts_pool: Arc<RwLock<Vec<LocalAccount>>>,
    ) -> Self {
        Self {
            creator,
            accounts_pool,
        }
    }
}

#[async_trait]
impl TransactionGenerator for AccountsPoolWrapperGenerator {
    fn generate_transactions(
        &mut self,
        accounts: Vec<&mut LocalAccount>,
        transactions_per_account: usize,
    ) -> Vec<SignedTransaction> {
        let needed = accounts.len() * transactions_per_account;

        let mut accounts_pool = self.accounts_pool.write();
        let num_in_pool = accounts_pool.len();
        if num_in_pool < needed {
            sample!(
                SampleRate::Duration(Duration::from_secs(10)),
                warn!("Cannot fetch enough accounts from pool, left in pool {}, needed {}", num_in_pool, needed);
            );
            return Vec::new();
        }
        let mut accounts_to_burn = accounts_pool
            .drain((num_in_pool - needed)..)
            .collect::<Vec<_>>();

        self.creator
            .generate_transactions(accounts_to_burn.iter_mut().collect(), 1)
    }
}

pub struct AccountsPoolWrapperCreator {
    creator: Box<dyn TransactionGeneratorCreator>,
    accounts_pool: Arc<RwLock<Vec<LocalAccount>>>,
}

impl AccountsPoolWrapperCreator {
    pub fn new(
        creator: Box<dyn TransactionGeneratorCreator>,
        accounts_pool: Arc<RwLock<Vec<LocalAccount>>>,
    ) -> Self {
        Self {
            creator,
            accounts_pool,
        }
    }
}

#[async_trait]
impl TransactionGeneratorCreator for AccountsPoolWrapperCreator {
    async fn create_transaction_generator(&mut self) -> Box<dyn TransactionGenerator> {
        Box::new(AccountsPoolWrapperGenerator::new(
            self.creator.create_transaction_generator().await,
            self.accounts_pool.clone(),
        ))
    }
}
