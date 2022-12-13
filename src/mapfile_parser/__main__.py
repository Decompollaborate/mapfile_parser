#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

import mapfile_parser


def mapfileParserMain():
    parser = argparse.ArgumentParser()

    parser.add_argument("map_file", help="Path to map file")

    args = parser.parse_args()

    mapPath = Path(args.map_file)

    mapFile = mapfile_parser.MapFile()
    mapFile.readMapFile(mapPath)

    mapFile.printAsCsv()
    # mapFile.printFunctionsCsv()


if __name__ == "__main__":
    mapfileParserMain()
