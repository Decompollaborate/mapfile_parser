/* SPDX-FileCopyrightText: © 2023-2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{file, found_symbol_info};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Segment {
    pub name: String,

    pub vram: u64,

    pub size: u64,

    pub vrom: u64,

    pub align: Option<u64>,

    pub files_list: Vec<file::File>,
}

impl Segment {
    pub fn new(name: String, vram: u64, size: u64, vrom: u64, align: Option<u64>) -> Self {
        Segment {
            name,
            vram,
            size,
            vrom,
            align,
            files_list: Vec::new(),
        }
    }

    pub fn filter_by_section_type(&self, section_type: &str) -> Self {
        let mut new_segment = self.clone_no_filelist();

        for file in &self.files_list {
            if file.section_type == section_type {
                new_segment.files_list.push(file.clone());
            }
        }

        new_segment
    }

    pub fn get_every_file_except_section_type(&self, section_type: &str) -> Self {
        let mut new_segment = self.clone_no_filelist();

        for file in &self.files_list {
            if file.section_type != section_type {
                new_segment.files_list.push(file.clone());
            }
        }

        new_segment
    }

    pub fn find_symbol_by_name(
        &self,
        sym_name: &str,
    ) -> Option<found_symbol_info::FoundSymbolInfo> {
        for file in &self.files_list {
            if let Some(sym) = file.find_symbol_by_name(sym_name) {
                return Some(found_symbol_info::FoundSymbolInfo::new_default(
                    file.clone(),
                    sym.clone(),
                ));
            }
        }
        None
    }

    pub fn find_symbol_by_vram_or_vrom(
        &self,
        address: u64,
    ) -> Option<found_symbol_info::FoundSymbolInfo> {
        for file in &self.files_list {
            if let Some((sym, offset)) = file.find_symbol_by_vram_or_vrom(address) {
                return Some(found_symbol_info::FoundSymbolInfo::new(
                    file.clone(),
                    sym.clone(),
                    offset,
                ));
            }
        }
        None
    }

    pub fn mix_folders(&self) -> Self {
        let mut new_segment = self.clone_no_filelist();

        let mut aux_dict = HashMap::new();

        // Put files in the same folder together
        for file in &self.files_list {
            // TODO: this is terrible
            let mut path: PathBuf = file
                .filepath
                .with_extension("")
                .components()
                .skip(2)
                .collect();
            path = path
                .components()
                .take(file.filepath.components().count() - 1)
                .collect();

            match aux_dict.entry(path) {
                Entry::Vacant(e) => {
                    e.insert(vec![file]);
                }
                Entry::Occupied(e) => {
                    e.into_mut().push(file);
                }
            }
        }

        // Pretend files in the same folder are one huge file
        for (folder_path, files_in_folder) in aux_dict.iter() {
            let first_file = files_in_folder[0];

            let vram = first_file.vram;
            let mut size = 0;
            let vrom = first_file.vrom;
            let section_type = &first_file.section_type;
            let align = first_file.align;

            let mut symbols = Vec::new();
            for file in files_in_folder {
                size += file.size;
                for sym in &file.symbols {
                    symbols.push(sym.clone());
                }
            }

            let mut temp_file =
                file::File::new(folder_path.clone(), vram, size, section_type, vrom, align);
            temp_file.symbols = symbols;
            new_segment.files_list.push(temp_file);
        }

        new_segment
    }

    pub fn fixup_non_matching_symbols(&mut self) {
        self.files_list
            .iter_mut()
            .for_each(|x| x.fixup_non_matching_symbols())
    }

    pub fn to_csv(&self, print_vram: bool, skip_without_symbols: bool) -> String {
        let mut ret = String::new();

        for file in &self.files_list {
            if skip_without_symbols && file.symbols.is_empty() {
                continue;
            }

            writeln!(ret, "{}", file.to_csv(print_vram)).unwrap();
        }

        ret
    }

    pub fn to_csv_symbols(&self) -> String {
        let mut ret = String::new();

        for file in &self.files_list {
            if file.symbols.is_empty() {
                continue;
            }

            for sym in &file.symbols {
                writeln!(ret, "{},{}", file.filepath.display(), sym.to_csv()).unwrap();
            }
        }

        ret
    }

    pub fn print_as_csv(&self, print_vram: bool, skip_without_symbols: bool) {
        print!("{}", self.to_csv(print_vram, skip_without_symbols));
    }

    pub fn print_symbols_csv(&self) {
        print!("{}", self.to_csv_symbols());
    }

    pub fn new_default(name: String, vram: u64, size: u64, vrom: u64) -> Self {
        Segment {
            name,
            vram,
            size,
            vrom,
            align: None,
            files_list: Vec::new(),
        }
    }

    pub fn clone_no_filelist(&self) -> Self {
        Segment {
            name: self.name.clone(),
            vram: self.vram,
            size: self.size,
            vrom: self.vrom,
            align: self.align,
            files_list: Vec::new(),
        }
    }

    pub fn new_placeholder() -> Self {
        Segment {
            name: "$nosegment".into(),
            vram: 0,
            size: 0,
            vrom: 0,
            align: None,
            files_list: vec![file::File::new_placeholder()],
        }
    }

    pub fn is_placeholder(&self) -> bool {
        if self.name == "$nosegment"
            && self.vram == 0
            && self.size == 0
            && self.vrom == 0
            && self.align.is_none()
        {
            if self.files_list.is_empty() {
                return true;
            }

            if self.files_list.len() == 1 {
                let first = self.files_list.first().unwrap();
                if first.is_placeholder() {
                    return true;
                }
            }
        }

        false
    }
}

// https://doc.rust-lang.org/std/cmp/trait.Eq.html
impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.vram == other.vram
            && self.size == other.size
            && self.vrom == other.vrom
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

#[cfg(feature = "python_bindings")]
#[allow(non_snake_case)]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use std::collections::hash_map::DefaultHasher;

    // Required to call the `.hash` and `.finish` methods, which are defined on traits.
    use std::hash::{Hash, Hasher};

    use crate::{file, found_symbol_info};

    #[pymethods]
    impl super::Segment {
        #[new]
        fn py_new(name: String, vram: u64, size: u64, vrom: u64, align: Option<u64>) -> Self {
            Self::new(name, vram, size, vrom, align)
        }

        /* Getters and setters */

        #[getter]
        fn get_name(&self) -> PyResult<String> {
            Ok(self.name.clone())
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
        fn get_size(&self) -> PyResult<u64> {
            Ok(self.size)
        }

        #[setter]
        fn set_size(&mut self, value: u64) -> PyResult<()> {
            self.size = value;
            Ok(())
        }

        #[getter]
        fn get_vrom(&self) -> PyResult<u64> {
            Ok(self.vrom)
        }

        #[setter]
        fn set_vrom(&mut self, value: u64) -> PyResult<()> {
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
        fn get_files_list(&self) -> PyResult<Vec<file::File>> {
            Ok(self.files_list)
        }

        #[setter]
        fn set_files_list(&mut self, value: Vec<file::File>) -> PyResult<()> {
            self.files_list = value;
            Ok(())
        }
        */

        /* Methods */

        fn filterBySectionType(&self, section_type: &str) -> Self {
            self.filter_by_section_type(section_type)
        }

        fn getEveryFileExceptSectionType(&self, section_type: &str) -> Self {
            self.get_every_file_except_section_type(section_type)
        }

        fn findSymbolByName(&self, sym_name: &str) -> Option<found_symbol_info::FoundSymbolInfo> {
            self.find_symbol_by_name(sym_name)
        }

        fn findSymbolByVramOrVrom(
            &self,
            address: u64,
        ) -> Option<found_symbol_info::FoundSymbolInfo> {
            self.find_symbol_by_vram_or_vrom(address)
        }

        fn mixFolders(&self) -> Self {
            self.mix_folders()
        }

        fn fixupNonMatchingSymbols(&mut self) {
            self.fixup_non_matching_symbols()
        }

        #[pyo3(signature=(print_vram=true, skip_without_symbols=true))]
        fn toCsv(&self, print_vram: bool, skip_without_symbols: bool) -> String {
            self.to_csv(print_vram, skip_without_symbols)
        }

        fn toCsvSymbols(&self) -> String {
            self.to_csv_symbols()
        }

        #[pyo3(signature=(print_vram=true, skip_without_symbols=true))]
        fn printAsCsv(&self, print_vram: bool, skip_without_symbols: bool) {
            self.print_as_csv(print_vram, skip_without_symbols)
        }

        fn printSymbolsCsv(&self) {
            self.print_symbols_csv()
        }

        fn copyFileList(&self) -> Vec<file::File> {
            self.files_list.clone()
        }

        fn setFileList(&mut self, new_list: Vec<file::File>) {
            self.files_list = new_list;
        }

        fn appendFile(&mut self, file: file::File) {
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
}
