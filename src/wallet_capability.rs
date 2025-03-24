use zewif::parser::prelude::*;
use anyhow::Result;
use std::sync::Arc;

use append_only_vec::AppendOnlyVec;
use zcash_client_backend::wallet::TransparentAddressMetadata;
use zcash_keys::address::UnifiedAddress;
use zcash_primitives::legacy::TransparentAddress;
use zingolib::{
    config::ChainType,
    wallet::{keys::unified::UnifiedKeyStore, traits::ReadableWriteable},
};

pub struct WalletCapability(zingolib::wallet::keys::unified::WalletCapability);

impl WalletCapability {
    pub fn unified_key_store(&self) -> &UnifiedKeyStore {
        &self.0.unified_key_store
    }

    pub fn transparent_child_addresses(&self) -> &Arc<AppendOnlyVec<(usize, TransparentAddress)>> {
        self.0.transparent_child_addresses()
    }

    pub fn rejection_addresses(
        &self,
    ) -> &Arc<AppendOnlyVec<(TransparentAddress, TransparentAddressMetadata)>> {
        self.0.get_rejection_addresses()
    }

    pub fn addresses(&self) -> &AppendOnlyVec<UnifiedAddress> {
        self.0.addresses()
    }
}

impl ParseWithParam<ChainType> for WalletCapability {
    fn parse(p: &mut Parser, param: ChainType) -> Result<Self> {
        Ok(Self(
            zingolib::wallet::keys::unified::WalletCapability::read(p, param)?,
        ))
    }
}

impl std::fmt::Debug for WalletCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugWalletCapability")
            .field("unified_key_store", self.unified_key_store())
            .field(
                "transparent_child_addresses",
                self.transparent_child_addresses(),
            )
            .field("rejection_addresses", self.rejection_addresses())
            .field("addresses", self.addresses())
            .finish()
    }
}

impl AsRef<zingolib::wallet::keys::unified::WalletCapability> for WalletCapability {
    fn as_ref(&self) -> &zingolib::wallet::keys::unified::WalletCapability {
        &self.0
    }
}
