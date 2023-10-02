/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{file, found_symbol_info};
use pyo3::prelude::*;
use pyo3::class::basic::CompareOp;
use std::collections::HashMap;
use std::fmt::Write;

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[pyclass(module = "mapfile_parser")]
pub struct Segment {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub vram: u64,

    #[pyo3(get, set)]
    pub size: u64,

    #[pyo3(get, set)]
    pub vrom: u64,

    // #[pyo3(get, set)]
    pub files_list: Vec<file::File>,
}

#[pymethods]
impl Segment {
    #[new]
    pub fn new(name: String, vram: u64, size: u64, vrom: u64) -> Self {
        Segment {
            name: name.into(),
            vram: vram,
            size: size,
            vrom: vrom,
            files_list: Vec::new(),
        }
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
        if humanReadable:
            return f"0x{self.vrom:06X}"
        return self.vrom
    */

    #[pyo3(name = "filterBySectionType")]
    pub fn filter_by_section_type(&self, section_type: &str) -> Segment {
        let mut new_segment = Segment::new(self.name.clone(), self.vram, self.size, self.vrom);

        for file in &self.files_list {
            if file.section_type == section_type {
                new_segment.files_list.push(file.clone());
            }
        }

        new_segment
    }

    #[pyo3(name = "getEveryFileExceptSectionType")]
    pub fn get_every_file_except_section_type(&self, section_type: &str) -> Segment {
        let mut new_segment = Segment::new(self.name.clone(), self.vram, self.size, self.vrom);

        for file in &self.files_list {
            if file.section_type != section_type {
                new_segment.files_list.push(file.clone());
            }
        }

        new_segment
    }

    #[pyo3(name = "findSymbolByName")]
    pub fn find_symbol_by_name(&self, sym_name: &str) -> Option<found_symbol_info::FoundSymbolInfo> {
        for file in &self.files_list {
            if let Some(sym) = file.find_symbol_by_name(sym_name) {
                return Some(found_symbol_info::FoundSymbolInfo::new_default(file.clone(), sym));
            }
        }
        None
    }

    #[pyo3(name = "findSymbolByVramOrVrom")]
    pub fn find_symbol_by_vram_or_vrom(&self, address: u64) -> Option<found_symbol_info::FoundSymbolInfo> {
        for file in &self.files_list {
            if let Some(pair) = file.find_symbol_by_vram_or_vrom(address) {
                let sym = pair.0;
                let offset = pair.1;

                return Some(found_symbol_info::FoundSymbolInfo::new(file.clone(), sym, offset));
            }
        }
        None
    }

    #[pyo3(name = "mixFolders")]
    pub fn mix_folders(&self) -> Segment {
        let mut new_segment = Segment::new(self.name.clone(), self.vram, self.size, self.vrom);

        // <PathBuf, Vec<File>>
        let mut aux_dict = HashMap::new();

        // Put files in the same folder together
        for file in &self.files_list {
            // TODO: this is terrible
            let mut path: PathBuf = file.filepath.with_extension("").components().skip(2).collect();
            path = path.components().take(file.filepath.components().count()-1).collect();

            if !aux_dict.contains_key(&path) {
                aux_dict.insert(path, vec![file]);
            } else {
                aux_dict.get_mut(&path).unwrap().push(file);
            }
        }

        // Pretend files in the same folder are one huge file
        for (folder_path, files_in_folder) in aux_dict.iter() {
            let first_file = files_in_folder[0];

            let vram = first_file.vram;
            let mut size = 0;
            let vrom = first_file.vrom;
            let section_type = &first_file.section_type;

            let mut symbols = Vec::new();
            for file in files_in_folder {
                size += file.size;
                for sym in &file.symbols {
                    symbols.push(sym.clone());
                }
            }

            let mut temp_file = file::File::new(folder_path.clone(), vram, size, section_type, vrom);
            temp_file.symbols = symbols;
            new_segment.files_list.push(temp_file);
        }

        new_segment
    }

    #[pyo3(name = "toCsv", signature=(print_vram=true, skip_without_symbols=true))]
    pub fn to_csv(&self, print_vram: bool, skip_without_symbols: bool) -> String {
        let mut ret = String::new();

        for file in &self.files_list {
            if skip_without_symbols && file.symbols.is_empty() {
                continue;
            }

            write!(ret, "{}\n", file.to_csv(print_vram)).unwrap();
        }

        ret
    }

    #[pyo3(name = "toCsvSymbols")]
    pub fn to_csv_symbols(&self) -> String {
        let mut ret = String::new();

        for file in &self.files_list {
            if file.symbols.is_empty() {
                continue;
            }

            for sym in &file.symbols {
                write!(ret, "{},{}\n", file.filepath.display(), sym.to_csv()).unwrap();
            }
        }

        ret
    }


    #[pyo3(name = "printAsCsv", signature=(print_vram=true, skip_without_symbols=true))]
    pub fn print_as_csv(&self, print_vram: bool, skip_without_symbols: bool) {
        print!("{}", self.to_csv(print_vram, skip_without_symbols));
    }

    #[pyo3(name = "printSymbolsCsv")]
    pub fn print_symbols_csv(&self) {
        print!("{}", self.to_csv_symbols());
    }

    /*
    def toJson(self, humanReadable: bool=True) -> dict[str, Any]:
        segmentDict: dict[str, Any] = {
            "name": self.name,
            "vram": self.serializeVram(humanReadable=humanReadable),
            "size": self.serializeSize(humanReadable=humanReadable),
            "vrom": self.serializeVrom(humanReadable=humanReadable),
        }

        filesList = []
        for file in self._filesList:
            filesList.append(file.toJson(humanReadable=humanReadable))

        segmentDict["files"] = filesList

        return segmentDict
    */

    #[pyo3(name = "copyFileList")]
    fn copy_file_list(&self) -> Vec<file::File> {
        self.files_list.clone()
    }

    #[pyo3(name = "setFileList")]
    fn set_file_list(&mut self, new_list: Vec<file::File>) {
        self.files_list = new_list;
    }

    #[pyo3(name = "appendFile")]
    fn append_file(&mut self, file: file::File) {
        self.files_list.push(file);
    }


    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<FileVecIter>> {
        let iter = FileVecIter {
            inner: slf.files_list.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }

    fn __getitem__(&self, index: usize) -> file::File {
        self.files_list[index].clone()
    }

    fn __setitem__(&mut self, index: usize, element: file::File) {
        self.files_list[index] = element;
    }

    fn __len__(&self) -> usize {
        self.files_list.len()
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

impl Segment {
    pub fn new_placeholder() -> Self {
        Segment {
            name: "$nosegment".into(),
            vram: 0,
            size: 0,
            vrom: 0,
            files_list: Vec::new(),
        }
    }

    pub fn is_placeholder(&self) -> bool {
        self.name == "$nosegment" && self.vram == 0 && self.size == 0 && self.files_list.is_empty()
    }
}

// https://doc.rust-lang.org/std/cmp/trait.Eq.html
impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.vram == other.vram && self.size == other.size && self.vrom == other.vrom
    }
}
impl Eq for Segment {}

// https://doc.rust-lang.org/std/hash/trait.Hash.html
impl Hash for Segment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.vram.hash(state);
        self.size.hash(state);
        self.vrom.hash(state);
    }
}

#[pyclass]
struct FileVecIter {
    inner: std::vec::IntoIter<file::File>,
}

#[pymethods]
impl FileVecIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<file::File> {
        slf.inner.next()
    }
}
