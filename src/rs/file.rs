/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::path::PathBuf;
use std::collections::hash_map::DefaultHasher;

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::hash::{Hash, Hasher};

use crate::symbol;
use pyo3::prelude::*;
use pyo3::class::basic::CompareOp;

#[derive(Debug, Clone)]
#[pyo3::prelude::pyclass(module = "mapfile_parser", unsendable, sequence)]
pub struct File {
    #[pyo3(get, set)]
    pub filepath: PathBuf,

    #[pyo3(get, set)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: u64,

    #[pyo3(get, set)]
    pub section_type: String,

    #[pyo3(get, set)]
    pub vrom: Option<u64>,

    #[pyo3(get, set)]
    pub symbols: Vec<symbol::Symbol>,
}

#[pyo3::prelude::pymethods]
impl File {
    #[new]
    pub fn new(filepath: std::path::PathBuf, vram: u64, size: u64, section_type: &str) -> Self {
        File {
            filepath: filepath,
            vram: vram,
            size: size,
            section_type: section_type.into(),
            vrom: None,
            symbols: Vec::new(),
        }
    }

    #[getter]
    #[pyo3(name = "getVramStr")]
    pub fn is_noload_section(&self) -> bool {
        return self.section_type == ".bss";
    }

    /*
    def serializeVram(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.vram:08X}"
        return self.vram

    def serializeSize(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.size:X}"
        return self.size

    def serializeVrom(self, humanReadable: bool=True) -> str|int|None:
        if self.vrom is None:
            return None
        if humanReadable:
            return f"0x{self.vrom:06X}"
        return self.vrom
    */


    // ! @deprecated
    #[pyo3(name = "getName")]
    fn get_name(&self) -> PathBuf {
        self.filepath.with_extension("").components().skip(2).collect()
    }
    //def getName(self) -> Path:
    //    return Path(*self.filepath.with_suffix("").parts[2:])


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

    // TODO: is this required?
    //def __iter__(self) -> Generator[Symbol, None, None]:
    //    for sym in self._symbols:
    //        yield sym

    fn __getitem__(&self, index: usize) -> symbol::Symbol {
        self.symbols[index].clone()
    }

    fn __setitem__(&mut self, index: usize, element: symbol::Symbol) {
        self.symbols[index] = element;
    }

    fn __len__(&self) -> usize {
        self.symbols.len()
    }

    // TODO: implement __eq__ instead when PyO3 0.20 releases
    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
        match op {
            pyo3::class::basic::CompareOp::Eq => (self.filepath == other.filepath).into_py(py),
            pyo3::class::basic::CompareOp::Ne => (self.filepath != other.filepath).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.filepath.hash(&mut hasher);
        hasher.finish() as isize
    }
}