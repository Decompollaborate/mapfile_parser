/* SPDX-FileCopyrightText: © 2023 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub fn parse_hex(src: &str) -> u64 {
    u64::from_str_radix(src.trim_start_matches("0x"), 16).unwrap()
}
