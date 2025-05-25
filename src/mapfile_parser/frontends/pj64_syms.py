#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import decomp_settings
from pathlib import Path
from typing import TextIO
import sys

from .. import mapfile


def writePj64SymsToFile(mapFile: mapfile.MapFile, outFile: TextIO):
    for segment in mapFile:
        for file in segment:
            for sym in file:
                symType = "code" if file.sectionType == ".text" else "data"
                outFile.write(f"{sym.vram:08X},{symType},{sym.name}\n")

def doPj64Syms(mapPath: Path, outputPath: Path|None) -> int:
    if not mapPath.exists():
        print(f"Could not find mapfile at '{mapPath}'")
        return 1

    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    if outputPath is None:
        writePj64SymsToFile(mapFile, sys.stdout)
    else:
        with outputPath.open("w") as outFile:
            writePj64SymsToFile(mapFile, outFile)

    return 0

def processArguments(args: argparse.Namespace, decompConfig: decomp_settings.Config|None=None):
    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        assert version is not None, f"Invalid version '{args.version}' selected"

        mapPath = Path(version.paths.map)
    else:
        mapPath = args.mapfile

    outputPath: Path = args.output

    exit(doPj64Syms(mapPath, outputPath))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser], decompConfig: decomp_settings.Config|None=None):
    parser = subparser.add_parser("pj64_syms", help="Produce a PJ64 compatible symbol map.")

    emitMapfile = True
    if decompConfig is not None:
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)

        if len(versions) > 0:
            parser.add_argument("-v", "--version", help="Version to process from the decomp.yaml file", type=str, choices=versions, default=versions[0])
            emitMapfile = False

    if emitMapfile:
        parser.add_argument("mapfile", help="Path to a map file.", type=Path)

    parser.add_argument("output", help="Path to output file. If omitted then output will be written to stdout", type=Path, nargs="?")

    parser.set_defaults(func=processArguments)
