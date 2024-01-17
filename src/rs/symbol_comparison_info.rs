/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{file, symbol};

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
pub struct SymbolComparisonInfo {
    pub symbol: symbol::Symbol,

    pub build_address: u64,

    pub build_file: Option<file::File>,

    pub expected_address: u64,

    pub expected_file: Option<file::File>,

    pub diff: Option<i64>,
}

impl SymbolComparisonInfo {
    pub fn new(
        symbol: symbol::Symbol,
        build_address: u64,
        build_file: Option<file::File>,
        expected_address: u64,
        expected_file: Option<file::File>,
        diff: Option<i64>,
    ) -> Self {
        Self {
            symbol,
            build_address,
            build_file,
            expected_address,
            expected_file,
            diff,
        }
    }
}

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use crate::{file, symbol};

    #[pymethods]
    impl super::SymbolComparisonInfo {
        #[new]
        #[pyo3(signature = (symbol, build_address, build_file, expected_address, expected_file, diff))]
        pub fn py_new(
            symbol: symbol::Symbol,
            build_address: u64,
            build_file: Option<file::File>,
            expected_address: u64,
            expected_file: Option<file::File>,
            diff: Option<i64>,
        ) -> Self {
            Self::new(
                symbol,
                build_address,
                build_file,
                expected_address,
                expected_file,
                diff,
            )
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
        fn get_buildFile(&self) -> PyResult<Option<file::File>> {
            Ok(self.build_file.clone())
        }
        #[setter]
        fn set_buildFile(&mut self, value: Option<file::File>) -> PyResult<()> {
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
        fn get_expectedFile(&self) -> PyResult<Option<file::File>> {
            Ok(self.expected_file.clone())
        }

        #[setter]
        fn set_expectedFile(&mut self, value: Option<file::File>) -> PyResult<()> {
            self.expected_file = value;
            Ok(())
        }

        #[getter]
        fn get_diff(&self) -> PyResult<Option<i64>> {
            Ok(self.diff)
        }

        #[setter]
        fn set_diff(&mut self, value: Option<i64>) -> PyResult<()> {
            self.diff = value;
            Ok(())
        }
    }
}
