#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from pathlib import Path


def convertibleToInt(value, base: int=10) -> bool:
    try:
        int(value, base)
        return True
    except ValueError:
        return False

def readFileAsBytearray(filepath: Path) -> bytearray:
    if not filepath.exists():
        return bytearray(0)
    with filepath.open(mode="rb") as f:
        return bytearray(f.read())

def hexbytes(bs):
    return ":".join("{:02X}".format(c) for c in bs)
