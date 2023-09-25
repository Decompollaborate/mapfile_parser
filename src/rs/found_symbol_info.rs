/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{file, symbol};

#[derive(Debug, Clone)]
#[pyo3::prelude::pyclass(module = "mapfile_parser", unsendable)]
pub struct FoundSymbolInfo {
    #[pyo3(get, set)]
    pub file: file::File,

    #[pyo3(get, set)]
    pub symbol: symbol::Symbol,

    #[pyo3(get, set)]
    pub offset: i32,
}

#[pyo3::prelude::pymethods]
impl FoundSymbolInfo {
    #[new]
    pub fn new(file: file::File, symbol: symbol::Symbol) -> Self {
        FoundSymbolInfo {
            file: file,
            symbol: symbol,
            offset: 0,
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
                message = format!("0x{0:X}", self.symbol.vram as i32 + self.offset);
            }
            message.push_str(&format!(" is at 0x{0:X} bytes inside", self.offset));
        } else {
            message = "Symbol".to_string();
        }

        format!("{0} {1}", message, self.get_as_str())
    }
}
