/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::fmt::Write;

use crate::{section, symbol};

#[derive(Debug, Clone)]
pub struct FoundSymbolInfo<'a> {
    pub section: &'a section::Section,

    pub symbol: &'a symbol::Symbol,

    pub offset: i64,
}

impl<'a> FoundSymbolInfo<'a> {
    pub fn new(section: &'a section::Section, symbol: &'a symbol::Symbol, offset: i64) -> Self {
        Self {
            section,
            symbol,
            offset,
        }
    }

    pub fn new_default(section: &'a section::Section, symbol: &'a symbol::Symbol) -> Self {
        Self {
            section,
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
            self.section.filepath.to_string_lossy()
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

    use crate::{section, symbol};

    #[derive(Debug, Clone)]
    #[pyclass(module = "mapfile_parser", name = "FoundSymbolInfo")]
    pub struct PyFoundSymbolInfo {
        pub section: section::Section,

        pub symbol: symbol::Symbol,

        pub offset: i64,
    }

    #[pymethods]
    impl PyFoundSymbolInfo {
        #[new]
        #[pyo3(signature=(section, symbol, offset=0))]
        fn new(section: section::Section, symbol: symbol::Symbol, offset: i64) -> Self {
            Self {
                section,
                symbol,
                offset,
            }
        }

        /* Getters and setters */

        #[getter]
        fn get_section(&self) -> PyResult<section::Section> {
            Ok(self.section.clone())
        }

        #[setter]
        fn set_section(&mut self, value: section::Section) -> PyResult<()> {
            self.section = value;
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
            let temp = super::FoundSymbolInfo::from(self);
            temp.get_as_str()
        }

        #[pyo3(name = "getAsStrPlusOffset")]
        #[pyo3(signature = (sym_name=None))]
        fn getAsStrPlusOffset(&self, sym_name: Option<String>) -> String {
            let temp = super::FoundSymbolInfo::from(self);
            temp.get_as_str_plus_offset(sym_name)
        }
    }

    impl<'a> From<&'a PyFoundSymbolInfo> for super::FoundSymbolInfo<'a> {
        fn from(value: &'a PyFoundSymbolInfo) -> Self {
            Self::new(&value.section, &value.symbol, value.offset)
        }
    }

    impl From<super::FoundSymbolInfo<'_>> for PyFoundSymbolInfo {
        fn from(value: super::FoundSymbolInfo) -> Self {
            Self::new(value.section.clone(), value.symbol.clone(), value.offset)
        }
    }
}
