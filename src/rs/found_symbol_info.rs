/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::borrow::Cow;

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
        self.get_as_str_impl(Cow::from(""))
    }

    pub(crate) fn get_as_str_impl(&self, extra: Cow<'_, str>) -> String {
        let name = &self.symbol.name;
        let vram = self.symbol.get_vram_str();
        let vrom = self.symbol.get_vrom_str();
        let size = self.symbol.get_size_str();
        let section_path = self.section.filepath.to_string_lossy();

        format!("'{name}' (VRAM: {vram}, VROM: {vrom}, SIZE: {size}, {section_path}{extra})")
    }

    pub fn get_as_str_plus_offset(&self, sym_name: Option<String>) -> String {
        self.get_as_str_plus_offset_impl(sym_name.map(Cow::from), Cow::from(""))
    }

    pub(crate) fn get_as_str_plus_offset_impl(
        &self,
        sym_name: Option<Cow<'_, str>>,
        extra: Cow<'_, str>,
    ) -> String {
        let message = if self.offset == 0 {
            Cow::from("Symbol")
        } else {
            let mes = if let Some(name) = sym_name {
                name
            } else {
                Cow::from(format!(
                    "0x{0:X}",
                    self.symbol.vram.wrapping_add_signed(self.offset)
                ))
            };
            Cow::from(format!("{} is at 0x{:X} bytes inside", mes, self.offset))
        };

        format!("{0} {1}", message, self.get_as_str_impl(extra))
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
