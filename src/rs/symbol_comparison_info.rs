/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{section, symbol};

#[derive(Debug, Clone)]
pub struct SymbolComparisonInfo<'a> {
    pub symbol: &'a symbol::Symbol,

    pub build_address: u64,

    pub build_file: Option<&'a section::Section>,

    pub expected_address: u64,

    pub expected_file: Option<&'a section::Section>,
}

impl<'a> SymbolComparisonInfo<'a> {
    pub fn new(
        symbol: &'a symbol::Symbol,
        build_address: u64,
        build_file: Option<&'a section::Section>,
        expected_address: u64,
        expected_file: Option<&'a section::Section>,
    ) -> Self {
        Self {
            symbol,
            build_address,
            build_file,
            expected_address,
            expected_file,
        }
    }

    pub fn diff(&self) -> Option<i64> {
        if self.build_address == u64::MAX {
            return None;
        }
        if self.expected_address == u64::MAX {
            return None;
        }

        let mut build_address = self.build_address;
        let mut expected_address = self.expected_address;

        // If both symbols are present in the same file then we do a diff
        // between their offsets into their respectives file.
        // This is done as a way to avoid too much noise in case an earlier file
        // did shift.
        if let Some(build_file) = &self.build_file {
            if let Some(expected_file) = &self.expected_file {
                if build_file.filepath == expected_file.filepath {
                    build_address -= build_file.vram;
                    expected_address -= expected_file.vram;
                }
            }
        }

        Some(build_address as i64 - expected_address as i64)
    }
}

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use crate::{section, symbol};

    #[derive(Debug, Clone)]
    #[pyclass(module = "mapfile_parser", name = "SymbolComparisonInfo")]
    pub struct PySymbolComparisonInfo {
        pub symbol: symbol::Symbol,

        pub build_address: u64,

        pub build_file: Option<section::Section>,

        pub expected_address: u64,

        pub expected_file: Option<section::Section>,
    }

    #[pymethods]
    impl PySymbolComparisonInfo {
        #[new]
        #[pyo3(signature = (symbol, build_address, build_file, expected_address, expected_file))]
        fn new(
            symbol: symbol::Symbol,
            build_address: u64,
            build_file: Option<section::Section>,
            expected_address: u64,
            expected_file: Option<section::Section>,
        ) -> Self {
            Self {
                symbol,
                build_address,
                build_file,
                expected_address,
                expected_file,
            }
        }

        /* Getters and setters */

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
        fn get_buildAddress(&self) -> PyResult<u64> {
            Ok(self.build_address)
        }
        #[setter]
        fn set_buildAddress(&mut self, value: u64) -> PyResult<()> {
            self.build_address = value;
            Ok(())
        }

        #[getter]
        fn get_buildFile(&self) -> PyResult<Option<section::Section>> {
            Ok(self.build_file.clone())
        }
        #[setter]
        fn set_buildFile(&mut self, value: Option<section::Section>) -> PyResult<()> {
            self.build_file = value;
            Ok(())
        }

        #[getter]
        fn get_expectedAddress(&self) -> PyResult<u64> {
            Ok(self.expected_address)
        }
        #[setter]
        fn set_expectedAddress(&mut self, value: u64) -> PyResult<()> {
            self.expected_address = value;
            Ok(())
        }

        #[getter]
        fn get_expectedFile(&self) -> PyResult<Option<section::Section>> {
            Ok(self.expected_file.clone())
        }

        #[setter]
        fn set_expectedFile(&mut self, value: Option<section::Section>) -> PyResult<()> {
            self.expected_file = value;
            Ok(())
        }

        #[pyo3(name = "diff")]
        fn py_diff(&self) -> PyResult<Option<i64>> {
            let temp = super::SymbolComparisonInfo::from(self);
            Ok(temp.diff())
        }
    }

    impl<'a> From<&'a PySymbolComparisonInfo> for super::SymbolComparisonInfo<'a> {
        fn from(value: &'a PySymbolComparisonInfo) -> Self {
            Self::new(
                &value.symbol,
                value.build_address,
                value.build_file.as_ref(),
                value.expected_address,
                value.expected_file.as_ref(),
            )
        }
    }

    impl From<super::SymbolComparisonInfo<'_>> for PySymbolComparisonInfo {
        fn from(value: super::SymbolComparisonInfo) -> Self {
            Self::new(
                value.symbol.clone(),
                value.build_address,
                value.build_file.cloned(),
                value.expected_address,
                value.expected_file.cloned(),
            )
        }
    }
}
