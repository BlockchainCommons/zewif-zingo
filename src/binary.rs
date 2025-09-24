use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use zewif::Data;

use crate::error::{ParseError, Result};

/// Thin wrapper around `Cursor` that provides convenience helpers for
/// length-checked reads and contextual error reporting.
#[derive(Debug)]
pub struct BinaryReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> BinaryReader<'a> {
    pub fn new(data: &'a Data) -> Self {
        Self { cursor: Cursor::new(data.as_slice()) }
    }

    pub fn remaining(&self) -> usize {
        let len = self.cursor.get_ref().len();
        let position = self.cursor.position() as usize;
        len.saturating_sub(position)
    }

    pub fn read_u8(&mut self, label: &'static str) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|source| ParseError::read(label, source))?;
        Ok(buf[0])
    }

    pub fn read_u32(&mut self, label: &'static str) -> Result<u32> {
        self.cursor
            .read_u32::<LittleEndian>()
            .map_err(|source| ParseError::read(label, source))
    }

    pub fn read_u64(&mut self, label: &'static str) -> Result<u64> {
        self.cursor
            .read_u64::<LittleEndian>()
            .map_err(|source| ParseError::read(label, source))
    }

    pub fn read_bool(&mut self, label: &'static str) -> Result<bool> {
        match self.read_u8(label)? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(ParseError::invalid_boolean(label, value)),
        }
    }

    pub fn read_string_with_u64_length(
        &mut self,
        label: &'static str,
    ) -> Result<String> {
        let length = self.read_u64(label)?;
        let length_usize = usize::try_from(length)
            .map_err(|_| ParseError::length_overflow(label, length))?;
        let bytes = self.read_exact_vec(label, length_usize)?;
        String::from_utf8(bytes)
            .map_err(|source| ParseError::InvalidString { label, source })
    }

    pub fn read_exact_vec(
        &mut self,
        label: &'static str,
        length: usize,
    ) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; length];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|source| ParseError::read(label, source))?;
        Ok(buf)
    }

    pub fn read_with<T, F>(&mut self, label: &'static str, op: F) -> Result<T>
    where
        F: FnOnce(&mut Cursor<&'a [u8]>) -> std::io::Result<T>,
    {
        op(&mut self.cursor).map_err(|source| ParseError::read(label, source))
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor<&'a [u8]> { &mut self.cursor }
}
