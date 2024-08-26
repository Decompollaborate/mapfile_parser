/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::path::PathBuf;

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::fmt::Write;
use std::hash::{Hash, Hasher};

use crate::{symbol, utils};

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct File {
    pub filepath: PathBuf,

    pub vram: u64,

    pub size: u64,

    pub section_type: String,

    pub vrom: Option<u64>,

    pub align: Option<u64>,

    pub symbols: Vec<symbol::Symbol>,
}

impl File {
    pub fn new(
        filepath: PathBuf,
        vram: u64,
        size: u64,
        section_type: &str,
        vrom: Option<u64>,
        align: Option<u64>,
    ) -> Self {
        Self {
            filepath,
            vram,
            size,
            section_type: section_type.into(),
            vrom,
            align,
            symbols: Vec::new(),
        }
    }

    pub fn is_noload_section(&self) -> bool {
        utils::is_noload_section(&self.section_type)
    }

    pub fn find_symbol_by_name(&self, sym_name: &str) -> Option<&symbol::Symbol> {
        self.symbols.iter().find(|&sym| sym.name == sym_name)
    }

    pub fn find_symbol_and_index_by_name(
        &self,
        sym_name: &str,
    ) -> Option<(&symbol::Symbol, usize)> {
        for (index, sym) in self.symbols.iter().enumerate() {
            if sym.name == sym_name {
                return Some((sym, index));
            }
        }
        None
    }

    pub fn find_symbol_by_name_mut(&mut self, sym_name: &str) -> Option<&mut symbol::Symbol> {
        self.symbols.iter_mut().find(|sym| sym.name == sym_name)
    }

    pub fn find_symbol_by_vram_or_vrom(&self, address: u64) -> Option<(&symbol::Symbol, i64)> {
        let mut prev_sym: Option<&symbol::Symbol> = None;

        let is_vram = address >= 0x1000000;

        for sym in &self.symbols {
            if sym.vram == address {
                return Some((sym, 0));
            }
            if let Some(sym_vrom_temp) = sym.vrom {
                if sym_vrom_temp == address {
                    return Some((sym, 0));
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
                            return Some((prev_sym_temp, offset));
                        }
                    }
                }
                if is_vram && sym.vram > address {
                    let offset = address as i64 - prev_sym_temp.vram as i64;
                    if offset < 0 {
                        return None;
                    }
                    return Some((prev_sym_temp, offset));
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
                        return Some((prev_sym_temp, offset));
                    }
                }

                if is_vram && prev_sym_temp.vram + prev_sym_temp_size > address {
                    let offset = address as i64 - prev_sym_temp.vram as i64;
                    if offset < 0 {
                        return None;
                    }
                    return Some((prev_sym_temp, offset));
                }
            }
        }

        None
    }

    pub fn fixup_non_matching_symbols(&mut self) {
        let mut symbols_to_fix = Vec::new();

        for (index, sym) in self.symbols.iter().enumerate() {
            if sym.name.ends_with(".NON_MATCHING") && sym.size.is_some() && sym.size == Some(0) {
                let real_name = sym.name.replace(".NON_MATCHING", "");

                if let Some((_real_sym, real_index)) =
                    self.find_symbol_and_index_by_name(&real_name)
                {
                    symbols_to_fix.push((real_index, sym.size));
                    symbols_to_fix.push((index, Some(0)));
                }
            }
        }

        for (index, new_size) in symbols_to_fix {
            if let Some(sym) = self.symbols.get_mut(index) {
                sym.size = new_size;
            }
        }
    }

    pub fn to_csv_header(print_vram: bool) -> String {
        let mut ret = String::new();

        if print_vram {
            ret.push_str("VRAM,");
        }
        ret.push_str("File,Section type,Num symbols,Max size,Total size,Average size");
        ret
    }

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

    pub fn print_csv_header(print_vram: bool) {
        println!("{}", Self::to_csv_header(print_vram));
    }

    pub fn print_as_csv(&self, print_vram: bool) {
        println!("{}", self.to_csv(print_vram));
    }
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
            align: None,
            symbols: Vec::new(),
        }
    }

    pub fn clone_no_symbollist(&self) -> Self {
        File {
            filepath: self.filepath.clone(),
            vram: self.vram,
            size: self.size,
            section_type: self.section_type.clone(),
            vrom: self.vrom,
            align: self.align,
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
            align: None,
            symbols: Vec::new(),
        }
    }

    pub fn is_placeholder(&self) -> bool {
        self.filepath.as_os_str().is_empty()
            && self.vram == 0
            && self.size == 0
            && self.section_type.is_empty()
            && self.vrom.is_none()
            && self.align.is_none()
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
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::{intern, prelude::*};

    use std::path::PathBuf;

    // Required to call the `.hash` and `.finish` methods, which are defined on traits.
    use std::hash::{Hash, Hasher};

    use crate::symbol;

    use std::collections::hash_map::DefaultHasher;

    #[pymethods]
    impl super::File {
        #[new]
        fn py_new(
            filepath: PathBuf,
            vram: u64,
            size: u64,
            section_type: &str,
            vrom: Option<u64>,
            align: Option<u64>,
        ) -> Self {
            Self::new(filepath, vram, size, section_type, vrom, align)
        }

        /* Getters and setters */

        // Manually convert PathBuf into a pathlib.Path object since pyo3 refuses to do so
        #[getter]
        fn get_filepath(&self) -> PyResult<PyObject> {
            Python::with_gil(|py| {
                let pathlib = py.import("pathlib")?;
                let pathlib_path = pathlib.getattr(intern!(py, "Path"))?;
                let args = (self.filepath.clone(),);

                Ok(pathlib_path.call1(args)?.to_object(py))
            })
        }

        #[setter]
        fn set_filepath(&mut self, value: PathBuf) -> PyResult<()> {
            self.filepath = value;
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
        fn get_size(&self) -> PyResult<u64> {
            Ok(self.size)
        }

        #[setter]
        fn set_size(&mut self, value: u64) -> PyResult<()> {
            self.size = value;
            Ok(())
        }

        #[getter]
        fn get_sectionType(&self) -> PyResult<String> {
            Ok(self.section_type.clone())
        }

        #[setter]
        fn set_sectionType(&mut self, value: String) -> PyResult<()> {
            self.section_type = value;
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

        /*
        #[getter]
        fn get__symbols(&self) -> PyResult<Vec<symbol::Symbol>> {
            Ok(self.symbols)
        }

        #[setter]
        fn set__symbols(&mut self, value: Vec<symbol::Symbol>) -> PyResult<()> {
            self.symbols = value;
            Ok(())
        }
        */

        #[getter]
        fn isNoloadSection(&self) -> bool {
            self.is_noload_section()
        }

        /* Methods */

        // ! @deprecated
        fn getName(&self) -> PathBuf {
            self.filepath
                .with_extension("")
                .components()
                .skip(2)
                .collect()
        }

        fn findSymbolByName(&self, sym_name: &str) -> Option<symbol::Symbol> {
            self.find_symbol_by_name(sym_name).cloned()
        }

        fn findSymbolByVramOrVrom(&self, address: u64) -> Option<(symbol::Symbol, i64)> {
            if let Some((sym, offset)) = self.find_symbol_by_vram_or_vrom(address) {
                Some((sym.clone(), offset))
            } else {
                None
            }
        }

        fn fixupNonMatchingSymbols(&mut self) {
            self.fixup_non_matching_symbols()
        }

        #[staticmethod]
        #[pyo3(signature=(print_vram=true))]
        fn toCsvHeader(print_vram: bool) -> String {
            Self::to_csv_header(print_vram)
        }

        #[pyo3(signature=(print_vram=true))]
        fn toCsv(&self, print_vram: bool) -> String {
            self.to_csv(print_vram)
        }

        #[staticmethod]
        #[pyo3(signature=(print_vram=true))]
        fn printCsvHeader(print_vram: bool) {
            Self::print_csv_header(print_vram)
        }

        #[pyo3(signature=(print_vram=true))]
        fn printAsCsv(&self, print_vram: bool) {
            self.print_as_csv(print_vram)
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

        fn copySymbolList(&self) -> Vec<symbol::Symbol> {
            self.symbols.clone()
        }

        fn setSymbolList(&mut self, new_list: Vec<symbol::Symbol>) {
            self.symbols = new_list;
        }

        fn appendSymbol(&mut self, sym: symbol::Symbol) {
            self.symbols.push(sym);
        }

        fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<SymbolVecIter>> {
            let iter = SymbolVecIter {
                inner: slf.symbols.clone().into_iter(),
            };
            Py::new(slf.py(), iter)
        }

        fn __getitem__(&self, index: usize) -> symbol::Symbol {
            self.symbols[index].clone()
        }

        fn __setitem__(&mut self, index: usize, element: symbol::Symbol) {
            self.symbols[index] = element;
        }

        fn __len__(&self) -> usize {
            self.symbols.len()
        }

        fn __eq__(&self, other: &Self) -> bool {
            self == other
        }

        fn __hash__(&self) -> isize {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish() as isize
        }

        // TODO: __str__ and __repr__
    }

    #[pyclass]
    struct SymbolVecIter {
        inner: std::vec::IntoIter<symbol::Symbol>,
    }

    #[pymethods]
    impl SymbolVecIter {
        fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<symbol::Symbol> {
            slf.inner.next()
        }
    }
}
