/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{file, symbol};
use std::fmt::Write;
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser")]
pub struct FoundSymbolInfo {
    #[pyo3(get, set)]
    pub file: file::File,

    #[pyo3(get, set)]
    pub symbol: symbol::Symbol,

    #[pyo3(get, set)]
    pub offset: i64,
}

#[pymethods]
impl FoundSymbolInfo {
    #[new]
    #[pyo3(signature=(file, symbol, offset=0))]
    pub fn new(file: file::File, symbol: symbol::Symbol, offset: i64) -> Self {
        FoundSymbolInfo {
            file,
            symbol,
            offset,
        }
    }

    #[pyo3(name = "getAsStr")]
    pub fn get_as_str(&self) -> String {
        format!("'{0}' (VRAM: {1}, VROM: {2}, SIZE: {3}, {4})", self.symbol.name, self.symbol.get_vram_str(), self.symbol.get_vrom_str(), self.symbol.get_size_str(), self.file.filepath.to_string_lossy())
    }

    #[pyo3(name = "getAsStrPlusOffset")]
    pub fn get_as_str_plus_offset(&self, sym_name: Option<String>) -> String {
        let mut message;

        if self.offset != 0 {
            if let Some(name) = sym_name {
                message = name;
            } else {
                message = format!("0x{0:X}", self.symbol.vram as i64 + self.offset);
            }
            write!(message, " is at 0x{0:X} bytes inside", self.offset).unwrap();
        } else {
            message = "Symbol".to_string();
        }

        format!("{0} {1}", message, self.get_as_str())
    }
}

impl FoundSymbolInfo {
    pub fn new_default(file: file::File, symbol: symbol::Symbol) -> Self {
        FoundSymbolInfo {
            file: file,
            symbol: symbol,
            offset: 0,
        }
    }
}
