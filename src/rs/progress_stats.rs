/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashMap;

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
pub struct ProgressStats {
    pub undecomped_size: usize,

    pub decomped_size: usize,
}

impl ProgressStats {
    pub fn new() -> Self {
        Self {
            undecomped_size: 0,
            decomped_size: 0,
        }
    }

    pub fn total(&self) -> usize {
        self.undecomped_size + self.decomped_size
    }

    pub fn undecomped_percentage(&self) -> f32 {
        self.undecomped_size as f32 / self.total() as f32 * 100.0
    }

    pub fn decomped_percentage(&self) -> f32 {
        self.decomped_size as f32 / self.total() as f32 * 100.0
    }

    pub fn undecomped_percentage_total(&self, total_stats: &Self) -> f32 {
        self.undecomped_size as f32 / total_stats.total() as f32 * 100.0
    }

    pub fn decomped_percentage_total(&self, total_stats: &Self) -> f32 {
        self.decomped_size as f32 / total_stats.total() as f32 * 100.0
    }

    pub fn get_as_frogress_entry(&self, name: &str) -> HashMap<String, usize> {
        let mut categories: HashMap<String, usize> = HashMap::new();

        categories.insert(name.to_string(), self.decomped_size);
        categories.insert(format!("{}/total", name), self.total());

        categories
    }

    pub fn get_header_as_str() -> String {
        format!(
            "{:<28}: {:>12} / {:>8} {:>10}%  ({:>20}%)",
            "Category", "DecompedSize", "Total", "OfFolder", "OfTotal"
        )
    }

    pub fn print_header() {
        println!("{}", Self::get_header_as_str());
    }

    pub fn get_entry_as_str(&self, category: &str, total_stats: &Self) -> String {
        format!(
            "{:<28}: {:>12} / {:>8} {:>10.4}%  ({:>8.4}% / {:>8.4}%)",
            category,
            self.decomped_size,
            self.total(),
            self.decomped_percentage(),
            self.decomped_percentage_total(total_stats),
            self.total() as f32 / total_stats.total() as f32 * 100.0
        )
    }

    pub fn print(&self, category: &str, total_stats: &Self) {
        println!("{}", self.get_entry_as_str(category, total_stats));
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
        fn get_undecompedSize(&self) -> PyResult<usize> {
            Ok(self.undecomped_size)
        }

        #[setter]
        fn set_undecompedSize(&mut self, value: usize) -> PyResult<()> {
            self.undecomped_size = value;
            Ok(())
        }

        #[getter]
        fn get_decompedSize(&self) -> PyResult<usize> {
            Ok(self.decomped_size)
        }

        #[setter]
        fn set_decompedSize(&mut self, value: usize) -> PyResult<()> {
            self.decomped_size = value;
            Ok(())
        }

        #[getter]
        #[pyo3(name = "total")]
        pub fn py_total(&self) -> usize {
            self.total()
        }

        /* Methods */

        fn undecompedPercentage(&self) -> f32 {
            self.undecomped_percentage()
        }

        fn decompedPercentage(&self) -> f32 {
            self.decomped_percentage()
        }

        fn undecompedPercentageTotal(&self, total_stats: &Self) -> f32 {
            self.undecomped_percentage_total(total_stats)
        }

        fn decompedPercentageTotal(&self, total_stats: &Self) -> f32 {
            self.decomped_percentage_total(total_stats)
        }

        pub fn getAsFrogressEntry(&self, name: &str) -> HashMap<String, usize> {
            self.get_as_frogress_entry(name)
        }

        #[staticmethod]
        pub fn getHeaderAsStr() -> String {
            Self::get_header_as_str()
        }

        #[staticmethod]
        pub fn printHeader() {
            Self::print_header()
        }

        pub fn getEntryAsStr(&self, category: &str, total_stats: &Self) -> String {
            self.get_entry_as_str(category, total_stats)
        }

        #[pyo3(name = "print")]
        pub fn py_print(&self, category: &str, total_stats: &Self) {
            self.print(category, total_stats)
        }
    }
}
