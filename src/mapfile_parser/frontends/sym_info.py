#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import mapfile
from .. import utils


def processArguments(args: argparse.Namespace):
    mapPath: Path = args.mapfile
    symName: str = args.symname

    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    if utils.convertibleToInt(symName, 0):
        info = mapFile.findSymbolByVramOrVrom(int(symName, 0))
    else:
        info = mapFile.findSymbolByName(symName)

    if info is None:
        print(f"'{symName}' not found in map file '{mapPath}'")
        return

    symFile, symbol, offset = info
    if offset != 0:
        message = f"{symName} is at 0x{offset:X} bytes inside"
    else:
        message = "Symbol"
    print(f"{message} '{symbol.name}' (VRAM: {symbol.getVramStr()}, VROM: {symbol.getVromStr()}, {symFile.filepath})")

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("sym_info", help="Display various information about a symbol or address.")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("symname", help="symbol name or VROM/VRAM address to lookup")

    parser.set_defaults(func=processArguments)
