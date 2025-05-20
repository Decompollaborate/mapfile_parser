/* SPDX-FileCopyrightText: © 2023-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

pub fn parse_hex(src: &str) -> u64 {
    u64::from_str_radix(src.trim_start_matches("0x"), 16).unwrap()
}

pub fn read_file_contents(file_path: &Path) -> String {
    let mut file_contents = String::new();
    let f = File::open(file_path).expect("Could not open input file");
    BufReader::new(f)
        .read_to_string(&mut file_contents)
        .expect("Not able to read the whole contents of the file");

    file_contents
}

pub fn is_noload_section(section_name: &str) -> bool {
    if section_name == ".bss" {
        return true;
    }
    if section_name == ".sbss" {
        return true;
    }
    if section_name == "COMMON" {
        return true;
    }
    if section_name == ".scommon" {
        return true;
    }

    false
}
