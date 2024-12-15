/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::hash::{Hash, Hasher};

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Symbol {
    pub name: String,

    pub vram: u64,

    pub size: Option<u64>,

    pub vrom: Option<u64>,

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

    pub fn get_align_str(&self) -> String {
        if let Some(align) = self.align {
            return format!("0x{:X}", align);
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
    use pyo3::{prelude::*, types::IntoPyDict, IntoPyObjectExt};

    use std::collections::hash_map::DefaultHasher;

    // Required to call the `.hash` and `.finish` methods, which are defined on traits.
    use std::hash::{Hash, Hasher};

    #[pymethods]
    impl super::Symbol {
        #[new]
        #[pyo3(signature=(name,vram,size=None,vrom=None,align=None))]
        fn py_new(
            name: String,
            vram: u64,
            size: Option<u64>,
            vrom: Option<u64>,
            align: Option<u64>,
        ) -> Self {
            Self::new(name, vram, size, vrom, align)
        }

        /* Getters and setters */

        #[getter]
        fn get_name(&self) -> PyResult<&str> {
            Ok(&self.name)
        }

        #[setter]
        fn set_name(&mut self, value: String) -> PyResult<()> {
            self.name = value;
            Ok(())
        }

        #[getter]
        fn get_vram(&self) -> PyResult<u64> {
            Ok(self.vram)
        }

        #[setter]
        fn set_vram(&mut self, value: u64) -> PyResult<()> {
            self.vram = value;
            Ok(())
        }

        #[getter]
        fn get_size(&self) -> PyResult<Option<u64>> {
            Ok(self.size)
        }

        #[setter]
        fn set_size(&mut self, value: Option<u64>) -> PyResult<()> {
            self.size = value;
            Ok(())
        }

        #[getter]
        fn get_vrom(&self) -> PyResult<Option<u64>> {
            Ok(self.vrom)
        }

        #[setter]
        fn set_vrom(&mut self, value: Option<u64>) -> PyResult<()> {
            self.vrom = value;
            Ok(())
        }

        #[getter]
        fn get_align(&self) -> PyResult<Option<u64>> {
            Ok(self.align)
        }

        #[setter]
        fn set_align(&mut self, value: Option<u64>) -> PyResult<()> {
            self.align = value;
            Ok(())
        }

        /* Serializers */

        #[pyo3(signature=(_humanReadable=true))]
        fn serializeName(&self, _humanReadable: bool) -> PyResult<PyObject> {
            Python::with_gil(|py| self.name.clone().into_py_any(py))
        }

        #[pyo3(signature=(humanReadable=true))]
        fn serializeVram(&self, humanReadable: bool) -> PyResult<PyObject> {
            Python::with_gil(|py| {
                if humanReadable {
                    return format!("0x{:08X}", self.vram).into_py_any(py);
                }

                self.vram.into_py_any(py)
            })
        }

        #[pyo3(signature=(humanReadable=true))]
        fn serializeSize(&self, humanReadable: bool) -> PyResult<PyObject> {
            Python::with_gil(|py| match self.size {
                None => Ok(Python::None(py)),
                Some(size) => {
                    if humanReadable {
                        return format!("0x{:X}", size).into_py_any(py);
                    }
                    size.into_py_any(py)
                }
            })
        }

        #[pyo3(signature=(humanReadable=true))]
        fn serializeVrom(&self, humanReadable: bool) -> PyResult<PyObject> {
            Python::with_gil(|py| match self.vrom {
                None => Ok(Python::None(py)),
                Some(vrom) => {
                    if humanReadable {
                        return format!("0x{:06X}", vrom).into_py_any(py);
                    }
                    vrom.into_py_any(py)
                }
            })
        }

        #[pyo3(signature=(humanReadable=true))]
        fn toJson(&self, humanReadable: bool) -> PyResult<PyObject> {
            Python::with_gil(|py| {
                [
                    ("name", self.serializeName(humanReadable)?),
                    ("vram", self.serializeVram(humanReadable)?),
                    ("size", self.serializeSize(humanReadable)?),
                    ("vrom", self.serializeVrom(humanReadable)?),
                ]
                .into_py_dict(py)?
                .into_py_any(py)
            })
        }

        /* Methods */

        fn getVramStr(&self) -> String {
            self.get_vram_str()
        }

        fn getSizeStr(&self) -> String {
            self.get_size_str()
        }

        fn getVromStr(&self) -> String {
            self.get_vrom_str()
        }

        fn getAlignStr(&self) -> String {
            self.get_align_str()
        }

        #[staticmethod]
        fn toCsvHeader() -> String {
            Self::to_csv_header()
        }

        fn toCsv(&self) -> String {
            self.to_csv()
        }

        #[staticmethod]
        fn printCsvHeader() {
            Self::print_csv_header()
        }

        fn printAsCsv(&self) {
            self.print_as_csv()
        }

        /* Python specific */

        fn __eq__(&self, other: &Self) -> bool {
            self == other
        }

        fn __hash__(&self) -> isize {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish() as isize
        }

        fn __repr__(&self) -> String {
            format!(
                "Symbol(name='{}', vram={}, size={}, vrom={}, align={})",
                self.name,
                self.get_vram_str(),
                self.get_size_str(),
                self.get_vrom_str(),
                self.get_align_str()
            )
        }
    }
}
