/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashSet;

use crate::{file, symbol_comparison_info};

#[derive(Debug, Clone)]
pub struct MapsComparisonInfo<'a> {
    pub bad_files: HashSet<&'a file::File>,

    pub missing_files: HashSet<&'a file::File>,

    pub compared_list: Vec<symbol_comparison_info::SymbolComparisonInfo<'a>>,
}

impl MapsComparisonInfo<'_> {
    pub fn new() -> Self {
        Self {
            bad_files: HashSet::new(),
            missing_files: HashSet::new(),
            compared_list: Vec::new(),
        }
    }
}

impl Default for MapsComparisonInfo<'_> {
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

    #[derive(Debug, Clone)]
    #[pyclass(module = "mapfile_parser", name = "MapsComparisonInfo")]
    pub struct PyMapsComparisonInfo {
        pub bad_files: HashSet<file::File>,

        pub missing_files: HashSet<file::File>,

        pub compared_list: Vec<symbol_comparison_info::python_bindings::PySymbolComparisonInfo>,
    }

    #[pymethods]
    impl PyMapsComparisonInfo {
        #[new]
        fn py_new() -> Self {
            Self {
                bad_files: HashSet::new(),
                missing_files: HashSet::new(),
                compared_list: Vec::new(),
            }
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
        fn get_comparedList(
            &self,
        ) -> PyResult<Vec<symbol_comparison_info::python_bindings::PySymbolComparisonInfo>>
        {
            Ok(self.compared_list.clone())
        }

        #[setter]
        fn set_comparedList(
            &mut self,
            value: Vec<symbol_comparison_info::python_bindings::PySymbolComparisonInfo>,
        ) -> PyResult<()> {
            self.compared_list = value;
            Ok(())
        }
    }

    impl<'a> From<&'a PyMapsComparisonInfo> for super::MapsComparisonInfo<'a> {
        fn from(value: &'a PyMapsComparisonInfo) -> Self {
            Self {
                bad_files: value.bad_files.iter().collect(),
                missing_files: value.missing_files.iter().collect(),
                compared_list: value
                    .compared_list
                    .iter()
                    .map(symbol_comparison_info::SymbolComparisonInfo::from)
                    .collect(),
            }
        }
    }

    impl From<super::MapsComparisonInfo<'_>> for PyMapsComparisonInfo {
        fn from(value: super::MapsComparisonInfo) -> Self {
            Self {
                bad_files: value.bad_files.into_iter().cloned().collect(),
                missing_files: value.missing_files.into_iter().cloned().collect(),
                compared_list: value
                    .compared_list
                    .into_iter()
                    .map(symbol_comparison_info::python_bindings::PySymbolComparisonInfo::from)
                    .collect(),
            }
        }
    }
}
