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
    mapFile = mapFile.filterBySegmentType(".text")

    if args.same_folder:
        mapFile = mapFile.mixFolders()

    if args.functions:
        mapFile.printSymbolsCsv()
    else:
        mapFile.printAsCsv(not args.same_folder)

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("function_sizes_csv", help="Produces a csv summarizing the files sizes by parsing a map file.")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("--same-folder", help="Mix files in the same folder.", action="store_true")
    parser.add_argument("--functions", help="Prints the size of every function instead of a summary.", action="store_true")

    parser.set_defaults(func=processArguments)
