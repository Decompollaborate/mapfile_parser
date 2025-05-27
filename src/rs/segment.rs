/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{
    collections::hash_map::{Entry, HashMap},
    fmt::Write,
    hash::{Hash, Hasher},
    path::PathBuf,
};

#[cfg(feature = "python_bindings")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{found_symbol_info, section};

#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "python_bindings", pyclass(module = "mapfile_parser"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Segment {
    pub name: String,

    pub vram: u64,

    pub size: u64,

    pub vrom: Option<u64>,

    pub align: Option<u64>,

    pub sections_list: Vec<section::Section>,
}

impl Segment {
    pub fn new(name: String, vram: u64, size: u64, vrom: Option<u64>, align: Option<u64>) -> Self {
        Segment {
            name,
            vram,
            size,
            vrom,
            align,
            sections_list: Vec::new(),
        }
    }

    pub fn filter_by_section_type(&self, section_type: &str) -> Self {
        let mut new_segment = self.clone_no_sectionlist();

        for section in &self.sections_list {
            if section.section_type == section_type {
                new_segment.sections_list.push(section.clone());
            }
        }

        new_segment
    }

    pub fn get_every_section_except_section_type(&self, section_type: &str) -> Self {
        let mut new_segment = self.clone_no_sectionlist();

        for section in &self.sections_list {
            if section.section_type != section_type {
                new_segment.sections_list.push(section.clone());
            }
        }

        new_segment
    }

    #[deprecated(
        since = "2.8.0",
        note = "Use `get_every_section_except_section_type` instead"
    )]
    pub fn get_every_file_except_section_type(&self, section_type: &str) -> Self {
        self.get_every_section_except_section_type(section_type)
    }

    pub fn find_symbol_by_name(
        &self,
        sym_name: &str,
    ) -> Option<found_symbol_info::FoundSymbolInfo> {
        for section in &self.sections_list {
            if let Some(sym) = section.find_symbol_by_name(sym_name) {
                return Some(found_symbol_info::FoundSymbolInfo::new_default(
                    section, sym,
                ));
            }
        }
        None
    }

    #[deprecated(
        since = "2.7.0",
        note = "Use `find_symbol_by_vram` or `find_symbol_by_vrom` instead."
    )]
    pub fn find_symbol_by_vram_or_vrom(
        &self,
        address: u64,
    ) -> Option<found_symbol_info::FoundSymbolInfo> {
        for section in &self.sections_list {
            #[allow(deprecated)]
            if let Some((sym, offset)) = section.find_symbol_by_vram_or_vrom(address) {
                return Some(found_symbol_info::FoundSymbolInfo::new(
                    section, sym, offset,
                ));
            }
        }
        None
    }

    pub fn find_symbol_by_vram(
        &self,
        address: u64,
    ) -> (
        Option<found_symbol_info::FoundSymbolInfo>,
        Vec<&section::Section>,
    ) {
        let mut possible_sections = Vec::new();
        for section in &self.sections_list {
            if let Some((sym, offset)) = section.find_symbol_by_vram(address) {
                return (
                    Some(found_symbol_info::FoundSymbolInfo::new(
                        section, sym, offset,
                    )),
                    Vec::new(),
                );
            }
            if address >= section.vram && address < section.vram + section.size {
                possible_sections.push(section);
            }
        }
        (None, possible_sections)
    }

    pub fn find_symbol_by_vrom(
        &self,
        address: u64,
    ) -> (
        Option<found_symbol_info::FoundSymbolInfo>,
        Vec<&section::Section>,
    ) {
        let mut possible_sections = Vec::new();
        for section in &self.sections_list {
            if let Some((sym, offset)) = section.find_symbol_by_vrom(address) {
                return (
                    Some(found_symbol_info::FoundSymbolInfo::new(
                        section, sym, offset,
                    )),
                    Vec::new(),
                );
            }
            if address >= section.vram && address < section.vram + section.size {
                possible_sections.push(section);
            }
        }
        (None, possible_sections)
    }

    pub fn mix_folders(&self) -> Self {
        let mut new_segment = self.clone_no_sectionlist();

        let mut aux_dict = HashMap::new();

        // Put sections in the same folder together
        for section in &self.sections_list {
            // TODO: this is terrible
            let mut path: PathBuf = section
                .filepath
                .with_extension("")
                .components()
                .skip(2)
                .collect();
            path = path
                .components()
                .take(section.filepath.components().count() - 1)
                .collect();

            match aux_dict.entry(path) {
                Entry::Vacant(e) => {
                    e.insert(vec![section]);
                }
                Entry::Occupied(e) => {
                    e.into_mut().push(section);
                }
            }
        }

        // Pretend sections in the same folder are one huge section
        for (folder_path, sections_in_folder) in aux_dict.iter() {
            let first_section = sections_in_folder[0];

            let vram = first_section.vram;
            let mut size = 0;
            let vrom = first_section.vrom;
            let section_type = &first_section.section_type;
            let align = first_section.align;

            let mut symbols = Vec::new();
            for section in sections_in_folder {
                size += section.size;
                for sym in &section.symbols {
                    symbols.push(sym.clone());
                }
            }

            let mut temp_section =
                section::Section::new(folder_path.clone(), vram, size, section_type, vrom, align);
            temp_section.symbols = symbols;
            new_segment.sections_list.push(temp_section);
        }

        new_segment
    }

    #[deprecated(
        since = "2.8.0",
        note = "This functionality is perform automatically during parsing now."
    )]
    pub fn fixup_non_matching_symbols(&mut self) {
        // This is a no-op now
    }

    pub fn to_csv(&self, print_vram: bool, skip_without_symbols: bool) -> String {
        let mut ret = String::new();

        for section in &self.sections_list {
            if skip_without_symbols && section.symbols.is_empty() {
                continue;
            }

            writeln!(ret, "{}", section.to_csv(print_vram)).unwrap();
        }

        ret
    }

    pub fn to_csv_symbols(&self) -> String {
        let mut ret = String::new();

        for section in &self.sections_list {
            if section.symbols.is_empty() {
                continue;
            }

            for sym in &section.symbols {
                writeln!(ret, "{},{}", section.filepath.display(), sym.to_csv()).unwrap();
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

    pub(crate) fn new_default(name: String, vram: u64, size: u64, vrom: Option<u64>) -> Self {
        Segment {
            name,
            vram,
            size,
            vrom,
            align: None,
            sections_list: Vec::new(),
        }
    }

    pub(crate) fn clone_no_sectionlist(&self) -> Self {
        Segment {
            name: self.name.clone(),
            vram: self.vram,
            size: self.size,
            vrom: self.vrom,
            align: self.align,
            sections_list: Vec::new(),
        }
    }

    pub fn new_placeholder() -> Self {
        Segment {
            name: "$nosegment".into(),
            vram: 0,
            size: 0,
            vrom: None,
            align: None,
            sections_list: vec![section::Section::new_placeholder()],
        }
    }

    pub(crate) fn is_placeholder(&self) -> bool {
        if self.name == "$nosegment"
            && self.vram == 0
            && self.size == 0
            && self.vrom.is_none()
            && self.align.is_none()
        {
            if self.sections_list.is_empty() {
                return true;
            }

            if self.sections_list.len() == 1 {
                let first = self.sections_list.first().unwrap();
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

    use crate::{found_symbol_info, section};

    #[pymethods]
    impl super::Segment {
        #[new]
        #[pyo3(signature = (name, vram, size, vrom, align=None))]
        fn py_new(
            name: String,
            vram: u64,
            size: u64,
            vrom: Option<u64>,
            align: Option<u64>,
        ) -> Self {
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

        /* Methods */

        fn filterBySectionType(&self, section_type: &str) -> Self {
            self.filter_by_section_type(section_type)
        }

        fn getEverySectionExceptSectionType(&self, section_type: &str) -> Self {
            self.get_every_section_except_section_type(section_type)
        }

        fn getEveryFileExceptSectionType(&self, section_type: &str) -> Self {
            self.getEverySectionExceptSectionType(section_type)
        }

        fn findSymbolByName(
            &self,
            sym_name: &str,
        ) -> Option<found_symbol_info::python_bindings::PyFoundSymbolInfo> {
            self.find_symbol_by_name(sym_name)
                .map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from)
        }

        fn findSymbolByVramOrVrom(
            &self,
            address: u64,
        ) -> Option<found_symbol_info::python_bindings::PyFoundSymbolInfo> {
            #[allow(deprecated)]
            self.find_symbol_by_vram_or_vrom(address)
                .map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from)
        }

        fn findSymbolByVram(
            &self,
            address: u64,
        ) -> (
            Option<found_symbol_info::python_bindings::PyFoundSymbolInfo>,
            Vec<section::Section>,
        ) {
            let (info, possible_sections) = self.find_symbol_by_vram(address);
            (
                info.map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from),
                possible_sections.into_iter().cloned().collect(),
            )
        }

        fn findSymbolByVrom(
            &self,
            address: u64,
        ) -> (
            Option<found_symbol_info::python_bindings::PyFoundSymbolInfo>,
            Vec<section::Section>,
        ) {
            let (info, possible_sections) = self.find_symbol_by_vrom(address);
            (
                info.map(found_symbol_info::python_bindings::PyFoundSymbolInfo::from),
                possible_sections.into_iter().cloned().collect(),
            )
        }

        fn mixFolders(&self) -> Self {
            self.mix_folders()
        }

        fn fixupNonMatchingSymbols(&mut self) {
            #[allow(deprecated)]
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

        fn copySectionList(&self) -> Vec<section::Section> {
            self.sections_list.clone()
        }
        fn setSectionList(&mut self, new_list: Vec<section::Section>) {
            self.sections_list = new_list;
        }
        fn appendSection(&mut self, section: section::Section) {
            self.sections_list.push(section);
        }

        fn copyFileList(&self) -> Vec<section::Section> {
            self.copySectionList()
        }
        fn setFileList(&mut self, new_list: Vec<section::Section>) {
            self.setSectionList(new_list)
        }
        fn appendFile(&mut self, section: section::Section) {
            self.appendSection(section)
        }

        fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<SectionVecIter>> {
            let iter = SectionVecIter {
                inner: slf.sections_list.clone().into_iter(),
            };
            Py::new(slf.py(), iter)
        }

        fn __getitem__(&self, index: usize) -> section::Section {
            self.sections_list[index].clone()
        }

        fn __setitem__(&mut self, index: usize, element: section::Section) {
            self.sections_list[index] = element;
        }

        fn __len__(&self) -> usize {
            self.sections_list.len()
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
    struct SectionVecIter {
        inner: std::vec::IntoIter<section::Section>,
    }

    #[pymethods]
    impl SectionVecIter {
        fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<section::Section> {
            slf.inner.next()
        }
    }
}
