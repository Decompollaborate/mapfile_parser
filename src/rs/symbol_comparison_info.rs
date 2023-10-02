/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pyo3::prelude::*;
use crate::{file, symbol};

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser")]
pub struct SymbolComparisonInfo {
    #[pyo3(get, set)]
    pub symbol: symbol::Symbol,

    #[pyo3(get, set, name="buildAddress")]
    pub build_address: u64,

    #[pyo3(get, set, name="buildFile")]
    pub build_file: Option<file::File>,

    #[pyo3(get, set, name="expectedAddress")]
    pub expected_address: u64,

    #[pyo3(get, set, name="expectedFile")]
    pub expected_file: Option<file::File>,

    #[pyo3(get, set)]
    pub diff: Option<i64>,
}

#[pymethods]
impl SymbolComparisonInfo {
    #[new]
    #[pyo3(signature = (symbol, build_address, build_file, expected_address, expected_file, diff))]
    pub fn new(symbol: symbol::Symbol, build_address: u64, build_file: Option<file::File>, expected_address: u64, expected_file: Option<file::File>, diff: Option<i64>) -> Self {
        SymbolComparisonInfo {
            symbol: symbol,
            build_address: build_address,
            build_file: build_file,
            expected_address: expected_address,
            expected_file: expected_file,
            diff: diff,
        }
    }
}
