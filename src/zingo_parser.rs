use std::io::{self, ErrorKind, Read};

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
use crate::{
    binary::BinaryReader,
    error::{ParseError, Result},
    WalletCapability,
    ZingoWallet,
};

#[derive(Debug)]
pub struct ZingoParser<'a> {
    reader: BinaryReader<'a>,
}

impl<'a> ZingoParser<'a> {
    const fn serialized_version() -> u64 {
        31
    }

    pub fn new(dump: &'a Data) -> Self {
        let reader = BinaryReader::new(dump);
        Self { reader }
    }

    pub fn parse(&mut self) -> Result<ZingoWallet> {
        let config = ZingoConfigBuilder::default().create();
        self.parse_with_param(config)
    }

    #[allow(unused_variables)]
    pub fn parse_with_param(&mut self, config: ZingoConfig) -> Result<ZingoWallet> {
        let reader = &mut self.reader;
        let external_version = reader.read_u64("external_version")?;
        if external_version > Self::serialized_version() {
            return Err(ParseError::UnsupportedWalletVersion {
                found: external_version,
                max: Self::serialized_version(),
            });
        }

        let wallet_capability =
            WalletCapability::read_from(reader.cursor_mut(), config.chain)?;

        let mut blocks = reader.read_with("BlockData", |cursor| {
            Vector::read(cursor, |r| BlockData::read(r))
        })?;
        if external_version <= 14 {
            // Reverse the order, since after version 20, we need highest-block-first
            // TODO: Consider order between 14 and 20.
            blocks = blocks.into_iter().rev().collect();
        }

        let transactions = if external_version <= 14 {
            reader.read_with("TxMap old", |cursor| {
                TxMap::read_old(cursor, wallet_capability.as_ref())
            })
        } else {
            reader.read_with("TxMap", |cursor| {
                TxMap::read(cursor, wallet_capability.as_ref())
            })
        }?;

        let chain_name = reader.read_string_with_u64_length("chain_name")?;

        let wallet_options = if external_version <= 23 {
            WalletOptions::default()
        } else {
            reader.read_with("WalletOptions", |cursor| WalletOptions::read(cursor))?
        };

        let birthday = reader.read_u64("birthday")?;

        if external_version <= 22 {
            let _sapling_tree_verified = if external_version <= 12 {
                true
            } else {
                reader.read_bool("sapling_tree_verified")?
            };
        }

        let verified_tree = if external_version <= 21 {
            None
        } else {
            reader.read_with("TreeState", |cursor| {
                Optional::read(cursor, |r| {
                    use prost::Message;

                    let buf = Vector::read(r, |reader| reader.read_u8())?;
                    TreeState::decode(&buf[..])
                        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))
                })
            })?
        };

        let price = if external_version <= 13 {
            WalletZecPriceInfo::default()
        } else {
            reader.read_with("WalletZecPriceInfo", |cursor| WalletZecPriceInfo::read(cursor))?
        };

        let _orchard_anchor_height_pairs = if external_version == 25 {
            reader.read_with("orchard_anchor_height_pairs", |cursor| {
                Vector::read(cursor, |r| {
                    let mut anchor_bytes = [0; 32];
                    r.read_exact(&mut anchor_bytes)?;
                    let block_height = BlockHeight::from_u32(r.read_u32::<LittleEndian>()?);
                    let anchor = Option::<orchard::Anchor>::from(
                        orchard::Anchor::from_bytes(anchor_bytes),
                    )
                    .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Bad orchard anchor"))?;
                    Ok((anchor, block_height))
                })
            })?
        } else {
            Vec::new()
        };

        let seed_bytes = reader.read_with("seed_bytes", |cursor| {
            Vector::read(cursor, |reader| reader.read_u8())
        })?;
        let mnemonic = if !seed_bytes.is_empty() {
            let account_index = if external_version >= 28 {
                reader.read_u32("account_index")?
            } else {
                0
            };
            Some((
                Mnemonic::from_entropy(seed_bytes).map_err(ParseError::from)?,
                account_index,
            ))
        } else {
            None
        };

        let remaining = reader.remaining();

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
