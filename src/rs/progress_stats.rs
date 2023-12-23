/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashMap;

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
pub struct ProgressStats {
    pub undecomped_size: u32,

    pub decomped_size: u32,
}

impl ProgressStats {
    pub fn new() -> Self {
        Self {
            undecomped_size: 0,
            decomped_size: 0,
        }
    }

    pub fn total(&self) -> u32 {
        self.undecomped_size + self.decomped_size
    }

    pub fn get_as_frogress_entry(&self, name: &str) -> HashMap<String, u32> {
        let mut categories: HashMap<String, u32> = HashMap::new();

        categories.insert(name.to_string(), self.decomped_size);
        categories.insert(format!("{}/total", name), self.total());

        categories
    }

    pub fn print_header() {
        println!(
            "{:<28}: {:>12} / {:>8} {:>10}%  ({:>20}%)",
            "Category", "DecompedSize", "Total", "OfFolder", "OfTotal"
        );
    }

    pub fn print(&self, category: &str, total_stats: &Self) {
        println!(
            "{:<28}: {:>12} / {:>8} {:>10.4}%  ({:>8.4}% / {:>8.4}%)",
            category,
            self.decomped_size,
            self.total(),
            self.decomped_size as f32 / self.total() as f32 * 100.0,
            self.decomped_size as f32 / total_stats.total() as f32 * 100.0,
            self.total() as f32 / total_stats.total() as f32 * 100.0
        );
    }
}

impl Default for ProgressStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use std::collections::HashMap;

    #[pymethods]
    impl super::ProgressStats {
        #[new]
        pub fn py_new() -> Self {
            Self::new()
        }

        /* Getters and setters */

        #[getter]
        fn get_undecompedSize(&self) -> PyResult<u32> {
            Ok(self.undecomped_size)
        }

        #[setter]
        fn set_undecompedSize(&mut self, value: u32) -> PyResult<()> {
            self.undecomped_size = value;
            Ok(())
        }

        #[getter]
        fn get_decompedSize(&self) -> PyResult<u32> {
            Ok(self.decomped_size)
        }

        #[setter]
        fn set_decompedSize(&mut self, value: u32) -> PyResult<()> {
            self.decomped_size = value;
            Ok(())
        }

        #[getter]
        #[pyo3(name = "total")]
        pub fn py_total(&self) -> u32 {
            self.total()
        }

        /* Methods */

        pub fn getAsFrogressEntry(&self, name: &str) -> HashMap<String, u32> {
            self.get_as_frogress_entry(name)
        }

        #[staticmethod]
        pub fn printHeader() {
            Self::print_header()
        }

        #[pyo3(name = "print")]
        pub fn py_print(&self, category: &str, total_stats: &Self) {
            self.print(category, total_stats)
        }
    }
}
