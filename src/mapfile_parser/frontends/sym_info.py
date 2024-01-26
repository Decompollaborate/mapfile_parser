#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import mapfile
from .. import utils


def doSymInfo(mapPath: Path, symName: str) -> int:
    if not mapPath.exists():
        print(f"Could not find mapfile at '{mapPath}'")
        return 1

    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    if utils.convertibleToInt(symName, 0):
        info = mapFile.findSymbolByVramOrVrom(int(symName, 0))
    else:
        info = mapFile.findSymbolByName(symName)

    if info is None:
        print(f"'{symName}' not found in map file '{mapPath}'")
        return 1
    print(info.getAsStrPlusOffset(symName))
    return 0


def processArguments(args: argparse.Namespace):
    mapPath: Path = args.mapfile
    symName: str = args.symname

    exit(doSymInfo(mapPath, symName))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("sym_info", help="Display various information about a symbol or address.")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("symname", help="symbol name or VROM/VRAM address to lookup")

    parser.set_defaults(func=processArguments)
