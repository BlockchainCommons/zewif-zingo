use std::io::{self, ErrorKind, Read};

use anyhow::{Context, Result, bail};
use bip0039::Mnemonic;
use byteorder::{LittleEndian, ReadBytesExt};
use zcash_client_backend::proto::service::TreeState;
use zcash_encoding::{Optional, Vector};
use zcash_protocol::consensus::BlockHeight;
use zingolib::{
    config::{ZingoConfig, ZingoConfigBuilder},
    wallet::{
        WalletOptions,
        data::{BlockData, WalletZecPriceInfo},
        tx_map::TxMap,
    },
};

use zewif::Data;
use zewif::{parse, parser::prelude::*};

use super::{WalletCapability, ZingoWallet};

#[derive(Debug)]
pub struct ZingoParser<'a> {
    pub parser: Parser<'a>,
}

impl<'a> ZingoParser<'a> {
    const fn serialized_version() -> u64 {
        31
    }

    pub fn new(dump: &'a Data) -> Self {
        let parser = Parser::new(dump);
        Self { parser }
    }

    pub fn parse(&mut self) -> Result<ZingoWallet> {
        let config = ZingoConfigBuilder::default().create();
        self.parse_with_param(config)
    }

    #[allow(unused_variables)]
    pub fn parse_with_param(&mut self, config: ZingoConfig) -> Result<ZingoWallet> {
        let p = &mut self.parser;
        // p.trace = true;
        let external_version = parse!(p, u64, "external_version")?;
        if external_version > Self::serialized_version() {
            bail!(
                "Don't know how to read wallet version {}. Do you have the latest version?",
                external_version
            );
        }

        let wallet_capability = parse!(
            p,
            WalletCapability,
            param = config.chain,
            "wallet_capability"
        )?;
        let mut blocks =
            Vector::read(&mut *p, |r| BlockData::read(r)).with_context(|| "BlockData")?;
        if external_version <= 14 {
            // Reverse the order, since after version 20, we need highest-block-first
            // TODO: Consider order between 14 and 20.
            blocks = blocks.into_iter().rev().collect();
        }

        let transactions = if external_version <= 14 {
            TxMap::read_old(&mut *p, wallet_capability.as_ref()).with_context(|| "TxMap old")
        } else {
            TxMap::read(&mut *p, wallet_capability.as_ref()).with_context(|| "TxMap")
        }?;

        let chain_name = parse_string::<u64>(&mut *p)?;

        let wallet_options = if external_version <= 23 {
            WalletOptions::default()
        } else {
            WalletOptions::read(&mut *p).with_context(|| "WalletOptions")?
        };

        // let birthday = p.read_u64::<LittleEndian>()?;
        let birthday = parse!(p, u64, "birthday")?;

        if external_version <= 22 {
            let _sapling_tree_verified = if external_version <= 12 {
                true
            } else {
                // reader.read_u8()? == 1
                parse!(p, u8, "sapling_tree_verified")? == 1
            };
        }

        let verified_tree = if external_version <= 21 {
            None
        } else {
            Optional::read(&mut *p, |r| {
                use prost::Message;

                let buf = Vector::read(r, |r| r.read_u8())?;
                TreeState::decode(&buf[..])
                    .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))
            })?
        };

        let price = if external_version <= 13 {
            WalletZecPriceInfo::default()
        } else {
            WalletZecPriceInfo::read(&mut *p)?
        };

        let _orchard_anchor_height_pairs = if external_version == 25 {
            Vector::read(&mut *p, |r| {
                let mut anchor_bytes = [0; 32];
                r.read_exact(&mut anchor_bytes)?;
                let block_height = BlockHeight::from_u32(r.read_u32::<LittleEndian>()?);
                Ok((
                    Option::<orchard::Anchor>::from(orchard::Anchor::from_bytes(anchor_bytes))
                        .ok_or(io::Error::new(ErrorKind::InvalidData, "Bad orchard anchor"))?,
                    block_height,
                ))
            })?
        } else {
            Vec::new()
        };

        let seed_bytes = Vector::read(&mut *p, |r| r.read_u8())?;
        let mnemonic = if !seed_bytes.is_empty() {
            let account_index = if external_version >= 28 {
                parse!(p, u32, "account_index")?
            } else {
                0
            };
            Some((
                Mnemonic::from_entropy(seed_bytes)
                    .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))?,
                account_index,
            ))
        } else {
            None
        };

        let remaining = p.remaining();

        Ok(ZingoWallet::new(
            external_version,
            chain_name,
            birthday,
            mnemonic,
            wallet_options,
            wallet_capability,
            verified_tree,
            price,
            blocks,
            remaining,
        ))
    }
}
