#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import mapfile_parser
from pathlib import Path

outputFolder = Path("tests/output/")
outputFolder.mkdir(parents=True, exist_ok=True)

for mapPath in Path("tests/maps").iterdir():
    print(mapPath)

    mapfile_parser.frontends.jsonify.doJsonify(mapPath, outputFolder/mapPath.with_suffix(".json").name)
    mapfile_parser.frontends.pj64_syms.doPj64Syms(mapPath, outputFolder/mapPath.with_suffix(".sym").name)
