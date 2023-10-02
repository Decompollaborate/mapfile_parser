/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashSet;

use crate::{file, symbol_comparison_info};
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser")]
pub struct MapsComparisonInfo {
    #[pyo3(get, set, name = "badFiles")]
    pub bad_files: HashSet<file::File>,

    #[pyo3(get, set, name = "missingFiles")]
    pub missing_files: HashSet<file::File>,

    #[pyo3(get, set, name = "comparedList")]
    pub compared_list: Vec<symbol_comparison_info::SymbolComparisonInfo>,
}

#[pymethods]
impl MapsComparisonInfo {
    #[new]
    pub fn new() -> Self {
        MapsComparisonInfo {
            bad_files: HashSet::new(),
            missing_files: HashSet::new(),
            compared_list: Vec::new(),
        }
    }
}
