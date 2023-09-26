/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::hash_map::DefaultHasher;

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::hash::{Hash, Hasher};

use pyo3::prelude::*;
use pyo3::class::basic::CompareOp;

#[derive(Debug, Clone)]
#[pyo3::prelude::pyclass(module = "mapfile_parser", unsendable)]
pub struct Symbol {
    #[pyo3(get)]
    pub name: String,

    #[pyo3(get)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: Option<u64>,

    #[pyo3(get, set)]
    pub vrom: Option<u64>,
}

#[pyo3::prelude::pymethods]
impl Symbol {
    #[new]
    pub fn new(name: String, vram: u64, size: Option<u64>, vrom: Option<u64>) -> Self {
        Symbol {
            name: name,
            vram: vram,
            size: size,
            vrom: vrom,
        }
    }

    #[pyo3(name = "getVramStr")]
    pub fn get_vram_str(&self) -> String {
        format!("0x{0:08X}", self.vram)
    }

    #[pyo3(name = "getSizeStr")]
    pub fn get_size_str(&self) -> String {
        if let Some(size) = self.size {
            return format!("0x{0:X}", size);
        }
        "None".into()
    }

    #[pyo3(name = "getVromStr")]
    pub fn get_vrom_str(&self) -> String {
        if let Some(vrom) = self.vrom {
            return format!("0x{0:06X}", vrom);
        }
        "None".into()
    }

    /*
    def serializeVram(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.vram:08X}"
        return self.vram

    def serializeSize(self, humanReadable: bool=True) -> str|int|None:
        if self.size is None:
            return None
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


    // doesn't work because Pyo3 doesn't support exporting enums to Python yet
    //#[pyo3(name = "serializeVrom")]
    //pub fn serialize_vrom(&self, human_readable: bool) -> Option<json_element::JsonElement> {
    //    if let Some(vrom) = self.vrom {
    //        if human_readable {
    //            return Some(json_element::JsonElement::String(format!("0x{:06X}", vrom)));
    //        }
    //        return Some(json_element::JsonElement::Int(vrom));
    //    }
    //    None
    //}


    #[staticmethod]
    #[pyo3(name = "toCsvHeader")]
    pub fn to_csv_header() -> String {
        "Symbol name,VRAM,Size in bytes".to_string()
    }

    #[pyo3(name = "toCsv")]
    pub fn to_csv(&self) -> String {
        format!("{0},{1:08X},{2}", self.name, self.vram, self.get_size_str())
    }


    #[staticmethod]
    #[pyo3(name = "printCsvHeader")]
    pub fn print_csv_header() {
        print!("{}", Self::to_csv_header());
    }

    #[pyo3(name = "printAsCsv")]
    pub fn print_as_csv(&self) {
        print!("{0}", self.to_csv());
    }


    /*
    def toJson(self, humanReadable: bool=True) -> dict[str, Any]:
        result: dict[str, Any] = {
            "name": self.name,
            "vram": self.serializeVram(humanReadable=humanReadable),
            "size": self.serializeSize(humanReadable=humanReadable),
            "vrom": self.serializeVrom(humanReadable=humanReadable),
        }

        return result
    */


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

impl Symbol {
    pub fn new_default(name: String, vram: u64) -> Self {
        Symbol {
            name: name,
            vram: vram,
            size: None,
            vrom: None,
        }
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
