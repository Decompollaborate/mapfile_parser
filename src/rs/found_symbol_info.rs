/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{file, symbol};
use std::fmt::Write;

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
pub struct FoundSymbolInfo {
    pub file: file::File,

    pub symbol: symbol::Symbol,

    pub offset: i64,
}

impl FoundSymbolInfo {
    pub fn new(file: file::File, symbol: symbol::Symbol, offset: i64) -> Self {
        Self {
            file,
            symbol,
            offset,
        }
    }

    pub fn new_default(file: file::File, symbol: symbol::Symbol) -> Self {
        Self {
            file,
            symbol,
            offset: 0,
        }
    }

    pub fn get_as_str(&self) -> String {
        format!(
            "'{0}' (VRAM: {1}, VROM: {2}, SIZE: {3}, {4})",
            self.symbol.name,
            self.symbol.get_vram_str(),
            self.symbol.get_vrom_str(),
            self.symbol.get_size_str(),
            self.file.filepath.to_string_lossy()
        )
    }

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

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use crate::{file, symbol};

    #[pymethods]
    impl super::FoundSymbolInfo {
        #[new]
        #[pyo3(signature=(file, symbol, offset=0))]
        fn py_new(file: file::File, symbol: symbol::Symbol, offset: i64) -> Self {
            Self::new(file, symbol, offset)
        }

        /* Getters and setters */

        #[getter]
        fn get_file(&self) -> PyResult<file::File> {
            Ok(self.file.clone())
        }

        #[setter]
        fn set_file(&mut self, value: file::File) -> PyResult<()> {
            self.file = value;
            Ok(())
        }

        #[getter]
        fn get_symbol(&self) -> PyResult<symbol::Symbol> {
            Ok(self.symbol.clone())
        }

        #[setter]
        fn set_symbol(&mut self, value: symbol::Symbol) -> PyResult<()> {
            self.symbol = value;
            Ok(())
        }

        #[getter]
        fn get_offset(&self) -> PyResult<i64> {
            Ok(self.offset)
        }

        #[setter]
        fn set_offset(&mut self, value: i64) -> PyResult<()> {
            self.offset = value;
            Ok(())
        }

        /* Methods */

        #[pyo3(name = "getAsStr")]
        fn getAsStr(&self) -> String {
            self.get_as_str()
        }

        #[pyo3(name = "getAsStrPlusOffset")]
        fn getAsStrPlusOffset(&self, sym_name: Option<String>) -> String {
            self.get_as_str_plus_offset(sym_name)
        }
    }
}
