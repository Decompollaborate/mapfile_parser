#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import mapfile


def processArguments(args: argparse.Namespace):
    mapFile = mapfile.MapFile()
    mapFile.readMapFile(args.mapfile)
    if args.filter_section is not None:
        mapFile = mapFile.filterBySegmentType(args.filter_section)

    if args.same_folder:
        mapFile = mapFile.mixFolders()

    if args.symbols:
        mapFile.printSymbolsCsv()
    else:
        mapFile.printAsCsv(printVram=not args.same_folder, skipWithoutSymbols=not args.all)

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("symbol_sizes_csv", help="Produces a csv summarizing the files sizes by parsing a map file.")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("--same-folder", help="Mix files in the same folder.", action="store_true")
    parser.add_argument("--symbols", help="Prints the size of every symbol instead of a summary.", action="store_true")
    parser.add_argument("-a", "--all", help="Don't skip files without symbols.", action="store_true")
    parser.add_argument("-f", "--filter-section", help="Only print the symbols of the passed section. For example: .text")

    parser.set_defaults(func=processArguments)
