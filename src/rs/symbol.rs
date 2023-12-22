/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::hash::{Hash, Hasher};

use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser")]
pub struct Symbol {
    #[pyo3(get)]
    pub name: String,

    #[pyo3(get)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: Option<u64>,

    #[pyo3(get, set)]
    pub vrom: Option<u64>,

    #[pyo3(get, set)]
    pub align: Option<u64>,
}

impl Symbol {
    pub fn new(
        name: String,
        vram: u64,
        size: Option<u64>,
        vrom: Option<u64>,
        align: Option<u64>,
    ) -> Self {
        Self {
            name,
            vram,
            size,
            vrom,
            align,
        }
    }

    pub fn new_default(name: String, vram: u64) -> Self {
        Self {
            name,
            vram,
            size: None,
            vrom: None,
            align: None,
        }
    }

    pub fn get_vram_str(&self) -> String {
        format!("0x{0:08X}", self.vram)
    }

    pub fn get_size_str(&self) -> String {
        if let Some(size) = self.size {
            //return format!("0x{0:X}", size);
            return format!("{}", size);
        }
        "None".into()
    }

    pub fn get_vrom_str(&self) -> String {
        if let Some(vrom) = self.vrom {
            return format!("0x{0:06X}", vrom);
        }
        "None".into()
    }

    pub fn to_csv_header() -> String {
        "Symbol name,VRAM,Size in bytes".to_string()
    }

    pub fn to_csv(&self) -> String {
        format!("{0},{1:08X},{2}", self.name, self.vram, self.get_size_str())
    }

    pub fn print_csv_header() {
        print!("{}", Self::to_csv_header());
    }

    pub fn print_as_csv(&self) {
        print!("{0}", self.to_csv());
    }
}

// https://doc.rust-lang.org/std/cmp/trait.Eq.html
impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.vram == other.vram
    }
}
impl Eq for Symbol {}

// https://doc.rust-lang.org/std/hash/trait.Hash.html
impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.vram.hash(state);
    }
}

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::class::basic::CompareOp;
    use pyo3::prelude::*;

    use std::collections::hash_map::DefaultHasher;

    // Required to call the `.hash` and `.finish` methods, which are defined on traits.
    use std::hash::{Hash, Hasher};

    #[pymethods]
    impl super::Symbol {
        #[new]
        pub fn py_new(
            name: String,
            vram: u64,
            size: Option<u64>,
            vrom: Option<u64>,
            align: Option<u64>,
        ) -> Self {
            Self::new(name,
                vram,
                size,
                vrom,
                align,
            )
        }

        pub fn getVramStr(&self) -> String {
            self.get_vram_str()
        }

        pub fn getSizeStr(&self) -> String {
            self.get_size_str()
        }

        pub fn getVromStr(&self) -> String {
            self.get_vrom_str()
        }

        #[staticmethod]
        pub fn toCsvHeader() -> String {
            Self::to_csv_header()
        }

        pub fn toCsv(&self) -> String {
            self.to_csv()
        }

        #[staticmethod]
        pub fn printCsvHeader() {
            Self::print_csv_header()
        }

        pub fn printAsCsv(&self) {
            self.print_as_csv()
        }

        // TODO: implement __eq__ instead when PyO3 0.20 releases
        fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
            match op {
                pyo3::class::basic::CompareOp::Eq => (self == other).into_py(py),
                pyo3::class::basic::CompareOp::Ne => (self != other).into_py(py),
                _ => py.NotImplemented(),
            }
        }

        fn __hash__(&self) -> isize {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish() as isize
        }

        // TODO: __str__ and __repr__
    }
}
