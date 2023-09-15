/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{vec, fs::File, io::{BufReader, Read}, path::PathBuf};

use regex;

use crate::utils;

#[derive(Debug, Clone)]
pub struct MapFile {
    // _segments_list: vec,
    pub debugging: bool,
}

impl MapFile {
    pub fn new() -> Self {
        MapFile {
            debugging: false,
        }
    }

    // TODO: look for equivalent to pathlib.Path
    pub fn read_map_file(&self, map_path: &String) {
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

        //for line in map_data.split(|&x| x==b'\n') {
        //    println!("{line:?}");
        //}

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
                    }
                }
            }

            if !in_file {
                if let Some(file_entry_match) = regex_fileDataEntry.captures(line) {
                    let filepath = std::path::PathBuf::from(&file_entry_match["name"]);
                    let size = utils::parse_hex(&file_entry_match["size"]);
                    let vram = utils::parse_hex(&file_entry_match["vram"]);
                    let section_type = &file_entry_match["section"];

                    if size > 0 {
                        in_file = true;
                    }

                    println!("filedata entry:");
                    println!("  filepath:     {filepath:?}");
                    println!("  size:         {size:X}");
                    println!("  vram:         {vram:X}");
                    println!("  section_type: {section_type}");
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
                } else if let Some(fill_match) = regex_fill.captures(line) {
                    // Make a dummy file to handle *fill*
                    let size = utils::parse_hex(&fill_match["size"]);
                    println!("fill info:");
                    println!("  {size:X}");
                }
            }

            prev_line = line;
        }

    }
}
