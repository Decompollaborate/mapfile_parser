/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashSet;

use crate::{section, symbol_comparison_info};

#[derive(Debug, Clone)]
pub struct MapsComparisonInfo<'a> {
    pub bad_sections: HashSet<&'a section::Section>,

    pub missing_sections: HashSet<&'a section::Section>,

    pub compared_list: Vec<symbol_comparison_info::SymbolComparisonInfo<'a>>,
}

impl MapsComparisonInfo<'_> {
    pub fn new() -> Self {
        Self {
            bad_sections: HashSet::new(),
            missing_sections: HashSet::new(),
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

    use crate::{section, symbol_comparison_info};

    #[derive(Debug, Clone)]
    #[pyclass(module = "mapsection_parser", name = "MapsComparisonInfo")]
    pub struct PyMapsComparisonInfo {
        pub bad_sections: HashSet<section::Section>,

        pub missing_sections: HashSet<section::Section>,

        pub compared_list: Vec<symbol_comparison_info::python_bindings::PySymbolComparisonInfo>,
    }

    #[pymethods]
    impl PyMapsComparisonInfo {
        #[new]
        fn py_new() -> Self {
            Self {
                bad_sections: HashSet::new(),
                missing_sections: HashSet::new(),
                compared_list: Vec::new(),
            }
        }

        /* Getters and setters */

        #[getter]
        fn get_badFiles(&self) -> PyResult<HashSet<section::Section>> {
            Ok(self.bad_sections.clone())
        }

        #[setter]
        fn set_badFiles(&mut self, value: HashSet<section::Section>) -> PyResult<()> {
            self.bad_sections = value;
            Ok(())
        }

        #[getter]
        fn get_missingFiles(&self) -> PyResult<HashSet<section::Section>> {
            Ok(self.missing_sections.clone())
        }

        #[setter]
        fn set_missingFiles(&mut self, value: HashSet<section::Section>) -> PyResult<()> {
            self.missing_sections = value;
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
                bad_sections: value.bad_sections.iter().collect(),
                missing_sections: value.missing_sections.iter().collect(),
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
                bad_sections: value.bad_sections.into_iter().cloned().collect(),
                missing_sections: value.missing_sections.into_iter().cloned().collect(),
                compared_list: value
                    .compared_list
                    .into_iter()
                    .map(symbol_comparison_info::python_bindings::PySymbolComparisonInfo::from)
                    .collect(),
            }
        }
    }
}
