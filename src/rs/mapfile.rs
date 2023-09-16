/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{vec, fs::File, io::{BufReader, Read}, path::PathBuf};

use regex;

use crate::{utils, segment, file, symbol};

#[derive(Debug, Clone)]
pub struct MapFile {
    segments_list: Vec<segment::Segment>,
    pub debugging: bool,
}

impl MapFile {
    pub fn new() -> Self {
        MapFile {
            segments_list: Vec::new(),
            debugging: false,
        }
    }

    // TODO: look for equivalent to pathlib.Path
    pub fn read_map_file(&mut self, map_path: &String) {
        // TODO: maybe move somewhere else?
        let regex_fileDataEntry = regex::Regex::new(r"^\s+(?P<section>\.[^\s]+)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<name>[^\s]+)$").unwrap();
        let regex_functionEntry = regex::Regex::new(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)$").unwrap();
        // regex_functionEntry = re.compile(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)((\s*=\s*(?P<expression>.+))?)$")
        let regex_label = regex::Regex::new(r"^(?P<name>\.?L[0-9A-F]{8})$").unwrap();
        let regex_fill = regex::Regex::new(r"^\s+(?P<fill>\*[^\s\*]+\*)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s*$").unwrap();
        let regex_segmentEntry = regex::Regex::new(r"(?P<name>([^\s]+)?)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<loadaddress>(load address)?)\s+(?P<vrom>0x[^\s]+)$").unwrap();



        let mut f = File::open(map_path).expect("Could not open input file");
        //let mut map_data: Vec<u8> = Vec::new();
        //let contents_length = f.read_to_end(&mut map_data).expect("Not able to read the whole contents of the file");
        let mut map_data = String::new();
        let contents_length = f.read_to_string(&mut map_data).expect("Not able to read the whole contents of the file");

        // TODO: "Linker script and memory map" stuff

        let mut temp_segment_list: Vec<segment::Segment> = Vec::new();
        temp_segment_list.push(segment::Segment::new(&"$nosegment".into(), 0, 0, 0));
        {
            let current_segment = temp_segment_list.last_mut().unwrap();

            current_segment.files_list.push(file::File::new(&"".into(), 0, 0, &"".into()));
        }

        let mut in_file = false;

        let mut prev_line = "";
        for line in map_data.split("\n") {
            println!("{line}");

            if in_file {
                if !line.starts_with("        ") {
                    in_file = false;
                } else if !regex_label.is_match(line) {
                    // Filter out jump table's labels

                    // Find symbols
                    if let Some(entry_match) = regex_functionEntry.captures(line) {
                        println!("{entry_match:?}");
                        let sym_name = &entry_match["name"];
                        let sym_vram = utils::parse_hex(&entry_match["vram"]);

                        println!("sym info:");
                        println!("  {sym_name}: {sym_vram:X}");

                        let current_segment = temp_segment_list.last_mut().unwrap();
                        let current_file = current_segment.files_list.last_mut().unwrap();

                        current_file.symbols.push(symbol::Symbol::new(&sym_name.into(), sym_vram));
                    }
                }
            }

            if !in_file {
                if let Some(file_entry_match) = regex_fileDataEntry.captures(line) {
                    let filepath = std::path::PathBuf::from(&file_entry_match["name"]);
                    let vram = utils::parse_hex(&file_entry_match["vram"]);
                    let size = utils::parse_hex(&file_entry_match["size"]);
                    let section_type = &file_entry_match["section"];

                    println!("filedata entry:");
                    println!("  filepath:     {filepath:?}");
                    println!("  size:         {size:X}");
                    println!("  vram:         {vram:X}");
                    println!("  section_type: {section_type}");

                    if size > 0 {
                        in_file = true;
                        let current_segment = temp_segment_list.last_mut().unwrap();

                        current_segment.files_list.push(file::File::new(&filepath, vram, size, &section_type.into()));
                    }
                } else if let Some(segment_entry_match) = regex_segmentEntry.captures(line) {
                    let mut name = &segment_entry_match["name"];
                    let vram = utils::parse_hex(&segment_entry_match["vram"]);
                    let size = utils::parse_hex(&segment_entry_match["size"]);
                    let vrom = utils::parse_hex(&segment_entry_match["vrom"]);

                    if name == "" {
                        // If the segment name is too long then this line gets break in two lines
                        name = prev_line;
                    }

                    println!("segment entry:");
                    println!("  name: {name}");
                    println!("  vram: {vram:X}");
                    println!("  size: {size:X}");
                    println!("  vrom: {vrom:X}");
                    temp_segment_list.push(segment::Segment::new(&name.into(), vram, size, vrom));
                    //current_segment = temp_segment_list.last_mut().unwrap();
                } else if let Some(fill_match) = regex_fill.captures(line) {
                    // Make a dummy file to handle *fill*
                    let mut filepath = std::path::PathBuf::new();
                    let mut vram = 0;
                    let mut size = utils::parse_hex(&fill_match["size"]);
                    let mut section_type = "".to_owned();

                    let current_segment = temp_segment_list.last_mut().unwrap();

                    if !current_segment.files_list.is_empty() {
                        let prev_file = current_segment.files_list.last().unwrap();
                        let mut name = prev_file.filepath.file_name().unwrap().to_owned();

                        name.push("__file__");
                        filepath = prev_file.filepath.with_file_name(name);
                        vram = prev_file.vram + prev_file.size;
                        section_type = prev_file.section_type.clone();
                    }

                    println!("fill info:");
                    println!("  {size:X}");

                    current_segment.files_list.push(file::File::new(&filepath, vram, size, &section_type));
                }
            }

            prev_line = line;
        }

        for i in 0..temp_segment_list.len() {
            let segment = &mut temp_segment_list[i];

            if i == 0 {
                if segment.size == 0 && segment.files_list.is_empty() {
                    // skip the dummy segment if it has no size, files or symbols
                    continue;
                }
            }

            let mut vrom_offset = segment.vrom;
            for file in segment.files_list.iter_mut() {
                let mut acummulated_size = 0;
                let symbols_count = file.symbols.len();
                let is_noload_section = file.is_noload_section();

                if file.vrom.is_some() {
                    vrom_offset = file.vrom.unwrap();
                }

                if !is_noload_section {
                    file.vrom = Some(vrom_offset);
                }

                if symbols_count > 0 {
                    let mut sym_vrom = vrom_offset;

                    // Calculate size of each symbol
                    for index in 0..symbols_count-1 {
                        let next_sym_vram = file.symbols[index+1].vram;
                        let sym = &mut file.symbols[index];

                        if index == 0 {
                            if sym.vram > file.vram {
                                // If the vram of the first symbol doesn't match the vram of the file
                                // it means the first(s) symbols were not emitted in the mapfile (static,
                                // jumptables, etc)
                                // We try to adjust the vrom to account for it.
                                sym_vrom += sym.vram - file.vram;
                            }
                        }

                        let sym_size = next_sym_vram - sym.vram;
                        acummulated_size += sym_size;

                        sym.size = Some(sym_size);

                        if !is_noload_section {
                            // Only set vrom of non bss variables
                            sym.vrom = Some(sym_vrom);
                            sym_vrom += sym_size;
                        }
                    }

                    // Calculate size of last symbol of the file
                    let sym = &mut file.symbols[symbols_count-1];
                    let sym_size = file.size - acummulated_size;
                    sym.size = Some(sym_size);
                    if !is_noload_section {
                        sym.vrom = Some(sym_vrom);
                        sym_vrom += sym_size;
                    }
                }

                if !is_noload_section {
                    vrom_offset += file.size;
                }
            }

            self.segments_list.push(segment.clone());
        }
    }
}
