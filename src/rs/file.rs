/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::path::PathBuf;

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::fmt::Write;
use std::hash::{Hash, Hasher};

use crate::{symbol, utils};

#[cfg(feature = "python_bindings")]
use std::collections::hash_map::DefaultHasher;
#[cfg(feature = "python_bindings")]
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser", sequence)]
pub struct File {
    #[pyo3(get, set, name = "_filepath_internal")]
    // TODO: pyo3 exposes this as str, need to fix somehow
    pub filepath: PathBuf,

    #[pyo3(get, set)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: u64,

    #[pyo3(get, set, name = "sectionType")]
    pub section_type: String,

    #[pyo3(get, set)]
    pub vrom: Option<u64>,

    // #[pyo3(get, set, name = "_symbols")]
    pub symbols: Vec<symbol::Symbol>,
}

#[pymethods]
impl File {
    #[new]
    pub fn new(
        filepath: PathBuf,
        vram: u64,
        size: u64,
        section_type: &str,
        vrom: Option<u64>,
    ) -> Self {
        File {
            filepath,
            vram,
            size,
            section_type: section_type.into(),
            vrom,
            symbols: Vec::new(),
        }
    }

    #[getter]
    #[pyo3(name = "isNoloadSection")]
    pub fn is_noload_section(&self) -> bool {
        utils::is_noload_section(&self.section_type)
    }

    #[cfg(feature = "python_bindings")]
    // ! @deprecated
    #[pyo3(name = "getName")]
    fn get_name(&self) -> PathBuf {
        self.filepath
            .with_extension("")
            .components()
            .skip(2)
            .collect()
    }

    #[pyo3(name = "findSymbolByName")]
    pub fn find_symbol_by_name(&self, sym_name: &str) -> Option<symbol::Symbol> {
        for sym in &self.symbols {
            if sym.name == sym_name {
                return Some(sym.clone());
            }
        }
        None
    }

    #[pyo3(name = "findSymbolByVramOrVrom")]
    pub fn find_symbol_by_vram_or_vrom(&self, address: u64) -> Option<(symbol::Symbol, i64)> {
        let mut prev_sym: Option<&symbol::Symbol> = None;

        let is_vram = address >= 0x1000000;

        for sym in &self.symbols {
            if sym.vram == address {
                return Some((sym.clone(), 0));
            }
            if let Some(sym_vrom_temp) = sym.vrom {
                if sym_vrom_temp == address {
                    return Some((sym.clone(), 0));
                }
            }

            if let Some(prev_sym_temp) = prev_sym {
                if let Some(sym_vrom) = sym.vrom {
                    if sym_vrom > address {
                        if let Some(prev_vrom_temp) = prev_sym_temp.vrom {
                            let offset = address as i64 - prev_vrom_temp as i64;
                            if offset < 0 {
                                return None;
                            }
                            return Some((prev_sym_temp.clone(), offset));
                        }
                    }
                }
                if is_vram && sym.vram > address {
                    let offset = address as i64 - prev_sym_temp.vram as i64;
                    if offset < 0 {
                        return None;
                    }
                    return Some((prev_sym_temp.clone(), offset));
                }
            }

            prev_sym = Some(sym);
        }

        if let Some(prev_sym_temp) = prev_sym {
            if let Some(prev_sym_temp_size) = prev_sym_temp.size {
                if let Some(prev_sym_temp_vrom) = prev_sym_temp.vrom {
                    if prev_sym_temp_vrom + prev_sym_temp_size > address {
                        let offset = address as i64 - prev_sym_temp_vrom as i64;
                        if offset < 0 {
                            return None;
                        }
                        return Some((prev_sym_temp.clone(), offset));
                    }
                }

                if is_vram && prev_sym_temp.vram + prev_sym_temp_size > address {
                    let offset = address as i64 - prev_sym_temp.vram as i64;
                    if offset < 0 {
                        return None;
                    }
                    return Some((prev_sym_temp.clone(), offset));
                }
            }
        }

        None
    }

    #[staticmethod]
    #[pyo3(name = "toCsvHeader", signature=(print_vram=true))]
    pub fn to_csv_header(print_vram: bool) -> String {
        let mut ret = String::new();

        if print_vram {
            ret.push_str("VRAM,");
        }
        ret.push_str("File,Section type,Num symbols,Max size,Total size,Average size");
        ret
    }

    #[pyo3(name = "toCsv", signature=(print_vram=true))]
    pub fn to_csv(&self, print_vram: bool) -> String {
        let mut ret = String::new();

        // Calculate stats
        let sym_count = self.symbols.len() as u64;
        let mut max_size = 0;
        let average_size = if sym_count > 0 {
            self.size as f64 / sym_count as f64
        } else {
            self.size as f64 / 1.0
        };

        for sym in &self.symbols {
            if let Some(sym_size) = sym.size {
                if sym_size > max_size {
                    max_size = sym_size;
                }
            }
        }

        if print_vram {
            //ret.push_str(format!("{:08X}", self.vram));
            write!(ret, "{:08X},", self.vram).unwrap();
            //ret += f"{self.vram:08X},";
        }
        write!(
            ret,
            "{},{},{},{},{},{:0.2}",
            self.filepath.display(),
            self.section_type,
            sym_count,
            max_size,
            self.size,
            average_size
        )
        .unwrap();

        ret
    }

    #[staticmethod]
    #[pyo3(name = "printCsvHeader", signature=(print_vram=true))]
    pub fn print_csv_header(print_vram: bool) {
        println!("{}", Self::to_csv_header(print_vram));
    }

    #[pyo3(name = "printAsCsv", signature=(print_vram=true))]
    pub fn print_as_csv(&self, print_vram: bool) {
        println!("{}", self.to_csv(print_vram));
    }

    /*
    def toJson(self, humanReadable: bool=True) -> dict[str, Any]:
        fileDict: dict[str, Any] = {
            "filepath": str(self.filepath),
            "sectionType": self.sectionType,
            "vram": self.serializeVram(humanReadable=humanReadable),
            "size": self.serializeSize(humanReadable=humanReadable),
            "vrom": self.serializeVrom(humanReadable=humanReadable),
        }

        symbolsList = []
        for symbol in self._symbols:
            symbolsList.append(symbol.toJson(humanReadable=humanReadable))

        fileDict["symbols"] = symbolsList
        return fileDict
    */

    #[cfg(feature = "python_bindings")]
    #[pyo3(name = "copySymbolList")]
    fn copy_symbol_list(&self) -> Vec<symbol::Symbol> {
        self.symbols.clone()
    }

    #[cfg(feature = "python_bindings")]
    #[pyo3(name = "setSymbolList")]
    fn set_symbol_list(&mut self, new_list: Vec<symbol::Symbol>) {
        self.symbols = new_list;
    }

    #[cfg(feature = "python_bindings")]
    #[pyo3(name = "appendSymbol")]
    fn append_symbol(&mut self, sym: symbol::Symbol) {
        self.symbols.push(sym);
    }

    #[cfg(feature = "python_bindings")]
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<SymbolVecIter>> {
        let iter = SymbolVecIter {
            inner: slf.symbols.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }

    #[cfg(feature = "python_bindings")]
    fn __getitem__(&self, index: usize) -> symbol::Symbol {
        self.symbols[index].clone()
    }

    #[cfg(feature = "python_bindings")]
    fn __setitem__(&mut self, index: usize, element: symbol::Symbol) {
        self.symbols[index] = element;
    }

    #[cfg(feature = "python_bindings")]
    fn __len__(&self) -> usize {
        self.symbols.len()
    }

    // TODO: implement __eq__ instead when PyO3 0.20 releases
    #[cfg(feature = "python_bindings")]
    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
        match op {
            pyo3::class::basic::CompareOp::Eq => (self == other).into_py(py),
            pyo3::class::basic::CompareOp::Ne => (self != other).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[cfg(feature = "python_bindings")]
    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    // TODO: __str__ and __repr__
}

impl File {
    pub fn new_default(
        filepath: std::path::PathBuf,
        vram: u64,
        size: u64,
        section_type: &str,
    ) -> Self {
        File {
            filepath,
            vram,
            size,
            section_type: section_type.into(),
            vrom: None,
            symbols: Vec::new(),
        }
    }

    pub fn new_placeholder() -> Self {
        Self {
            filepath: "".into(),
            vram: 0,
            size: 0,
            section_type: "".into(),
            vrom: None,
            symbols: Vec::new(),
        }
    }

    pub fn is_placeholder(&self) -> bool {
        self.filepath.as_os_str().is_empty()
            && self.vram == 0
            && self.size == 0
            && self.section_type.is_empty()
            && self.vrom.is_none()
            && self.symbols.is_empty()
    }
}

// https://doc.rust-lang.org/std/cmp/trait.Eq.html
impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.filepath == other.filepath
    }
}
impl Eq for File {}

// https://doc.rust-lang.org/std/hash/trait.Hash.html
impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filepath.hash(state);
    }
}

#[cfg(feature = "python_bindings")]
#[pyclass]
struct SymbolVecIter {
    inner: std::vec::IntoIter<symbol::Symbol>,
}

#[cfg(feature = "python_bindings")]
#[pymethods]
impl SymbolVecIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<symbol::Symbol> {
        slf.inner.next()
    }
}
