#![allow(dead_code)]

use bip0039::Mnemonic;
use zcash_client_backend::proto::service::TreeState;
use zingolib::wallet::{
    WalletOptions,
    data::{BlockData, WalletZecPriceInfo},
};

use super::WalletCapability;

#[derive(Debug)]
pub struct ZingoWallet {
    external_version: u64,
    chain_name: String,
    birthday: u64,
    mnemonic: Option<(Mnemonic, u32)>,
    wallet_options: WalletOptions,
    wallet_capability: WalletCapability,
    verified_tree: Option<TreeState>,
    price: WalletZecPriceInfo,
    last_100_blocks: Vec<BlockData>,
    remaining: usize,
}

impl ZingoWallet {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        external_version: u64,
        chain_name: String,
        birthday: u64,
        mnemonic: Option<(Mnemonic, u32)>,
        wallet_options: WalletOptions,
        wallet_capability: WalletCapability,
        verified_tree: Option<TreeState>,
        price: WalletZecPriceInfo,
        last_100_blocks: Vec<BlockData>,
        remaining: usize,
    ) -> Self {
        ZingoWallet {
            external_version,
            chain_name,
            birthday,
            mnemonic,
            wallet_options,
            wallet_capability,
            verified_tree,
            price,
            last_100_blocks,
            remaining,
        }
    }

    pub fn external_version(&self) -> u64 {
        self.external_version
    }

    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }

    pub fn birthday(&self) -> u64 {
        self.birthday
    }

    pub fn mnemonic(&self) -> &Option<(Mnemonic, u32)> {
        &self.mnemonic
    }

    pub fn wallet_options(&self) -> &WalletOptions {
        &self.wallet_options
    }

    pub fn wallet_capability(&self) -> &WalletCapability {
        &self.wallet_capability
    }

    pub fn verified_tree(&self) -> &Option<TreeState> {
        &self.verified_tree
    }

    pub fn price(&self) -> &WalletZecPriceInfo {
        &self.price
    }

    pub fn last_100_blocks(&self) -> &Vec<BlockData> {
        &self.last_100_blocks
    }

    pub fn remaining(&self) -> usize {
        self.remaining
    }
}
