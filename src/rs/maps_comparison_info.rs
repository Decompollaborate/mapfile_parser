/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashSet;

use crate::{file, symbol_comparison_info};

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
pub struct MapsComparisonInfo {
    pub bad_files: HashSet<file::File>,

    pub missing_files: HashSet<file::File>,

    pub compared_list: Vec<symbol_comparison_info::SymbolComparisonInfo>,
}

impl MapsComparisonInfo {
    pub fn new() -> Self {
        Self {
            bad_files: HashSet::new(),
            missing_files: HashSet::new(),
            compared_list: Vec::new(),
        }
    }
}

impl Default for MapsComparisonInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use std::collections::HashSet;

    use crate::{file, symbol_comparison_info};

    #[pymethods]
    impl super::MapsComparisonInfo {
        #[new]
        fn py_new() -> Self {
            Self::new()
        }

        /* Getters and setters */

        #[getter]
        fn get_badFiles(&self) -> PyResult<HashSet<file::File>> {
            Ok(self.bad_files.clone())
        }

        #[setter]
        fn set_badFiles(&mut self, value: HashSet<file::File>) -> PyResult<()> {
            self.bad_files = value;
            Ok(())
        }

        #[getter]
        fn get_missingFiles(&self) -> PyResult<HashSet<file::File>> {
            Ok(self.missing_files.clone())
        }

        #[setter]
        fn set_missingFiles(&mut self, value: HashSet<file::File>) -> PyResult<()> {
            self.missing_files = value;
            Ok(())
        }

        #[getter]
        fn get_comparedList(&self) -> PyResult<Vec<symbol_comparison_info::SymbolComparisonInfo>> {
            Ok(self.compared_list.clone())
        }

        #[setter]
        fn set_comparedList(
            &mut self,
            value: Vec<symbol_comparison_info::SymbolComparisonInfo>,
        ) -> PyResult<()> {
            self.compared_list = value;
            Ok(())
        }
    }
}
