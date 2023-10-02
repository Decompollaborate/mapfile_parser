/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::HashMap;

use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser", sequence)]
pub struct ProgressStats {
    #[pyo3(get, set, name = "undecompedSize")]
    pub undecomped_size: u32,

    #[pyo3(get, set, name = "decompedSize")]
    pub decomped_size: u32,
}

#[pymethods]
impl ProgressStats {
    #[new]
    pub fn new() -> Self {
        Self {
            undecomped_size: 0,
            decomped_size: 0,
        }
    }

    #[getter]
    pub fn total(&self) -> u32 {
        self.undecomped_size + self.decomped_size
    }

    #[pyo3(name = "getAsFrogressEntry")]
    pub fn get_as_frogress_entry(&self, name: &str) -> HashMap<String, u32> {
        let mut categories: HashMap<String, u32> = HashMap::new();

        categories.insert(name.to_string(), self.decomped_size);
        categories.insert(format!("{}/total", name), self.total());

        categories
    }

    #[staticmethod]
    #[pyo3(name = "printHeader")]
    pub fn print_header() {
        println!(
            "{:<28}: {:>12} / {:>8} {:>10}%  ({:>20}%)",
            "Category", "DecompedSize", "Total", "OfFolder", "OfTotal"
        );
    }

    pub fn print(&self, category: &str, total_stats: &ProgressStats) {
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
