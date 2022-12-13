#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import mapfile
from .. import utils


def doFirstDiff(mapPath: Path, expectedMapPath: Path, romPath: Path, expectedRomPath: Path, diffCount: int=5, mismatchSize: bool=False) -> int:
    if not mapPath.exists():
        print(f"{mapPath} must exist")
        return 1
    if not expectedMapPath.exists():
        print(f"{expectedMapPath} must exist")
        return 1
    if not romPath.exists():
        print(f"{romPath} must exist")
        return 1
    if not expectedRomPath.exists():
        print(f"{expectedRomPath} must exist")
        return 1

    builtRom = utils.readFileAsBytearray(romPath)
    expectedRom = utils.readFileAsBytearray(expectedRomPath)

    if len(builtRom) != len(expectedRom):
        print("Modified ROM has different size...")
        print(f"It should be 0x{len(expectedRom):X} but it is 0x{len(builtRom):X}")
        if not mismatchSize:
            return 1

    if builtRom == expectedRom:
        print("No differences!")
        return 0

    builtMapFile = mapfile.MapFile()
    builtMapFile.readMapFile(mapPath)
    expectedMapFile = mapfile.MapFile()
    expectedMapFile.readMapFile(expectedMapPath)

    map_search_diff: set[str] = set()
    diffs = 0
    shift_cap = 1000
    for i in range(24, min(len(builtRom), len(expectedRom)), 4):
        # (builtRom[i:i+4] != expectedRom[i:i+4], but that's slightly slower in CPython...)
        if diffs <= shift_cap and (
            builtRom[i] != expectedRom[i]
            or builtRom[i + 1] != expectedRom[i + 1]
            or builtRom[i + 2] != expectedRom[i + 2]
            or builtRom[i + 3] != expectedRom[i + 3]
        ):
            if diffs == 0:
                vromInfo = builtMapFile.findSymbolByVramOrVrom(i)
                extraMessage = ""
                if vromInfo is not None:
                    extraMessage = f", {vromInfo.getAsStrPlusOffset()}"
                print(f"First difference at ROM addr 0x{i:X}{extraMessage}")
                print(f"Bytes: {utils.hexbytes(builtRom[i : i + 4])} vs {utils.hexbytes(expectedRom[i : i + 4])}")
            diffs += 1

        if (
            len(map_search_diff) < diffCount
            and builtRom[i] >> 2 != expectedRom[i] >> 2
        ):
            vromInfo = builtMapFile.findSymbolByVramOrVrom(i)
            if vromInfo is not None:
                vromMessage = vromInfo.getAsStr()
                if vromMessage not in map_search_diff:
                    map_search_diff.add(vromMessage)

                    extraMessage = ""
                    if vromInfo is not None:
                        extraMessage = f", {vromInfo.getAsStrPlusOffset()}"
                    print(f"Instruction difference at ROM addr 0x{i:X}{extraMessage}")
                    print(f"Bytes: {utils.hexbytes(builtRom[i : i + 4])} vs {utils.hexbytes(expectedRom[i : i + 4])}")

        if len(map_search_diff) >= diffCount and diffs > shift_cap:
            break

    if diffs == 0:
        print("No differences but ROMs differ?")
        return 1

    print()
    definite_shift = diffs > shift_cap
    if definite_shift:
        print(f"Over {shift_cap} differing words, must be a shifted ROM.")
    else:
        print(f"{diffs} differing word(s).")

    if diffs > 100:
        firstDifferingSym = builtMapFile.findLowestDifferingSymbol(expectedMapFile)
        if firstDifferingSym is None:
            print(f"No ROM shift{' (!?)' if definite_shift else ''}")
        else:
            sym, file, prevSym = firstDifferingSym
            extraMessage = ""
            if prevSym is not None:
                extraMessage = f" -- in {prevSym.name}?"
            print(f"Map appears to have shifted just before {sym.name} ({file.filepath}){extraMessage}")
            return 1

    return 0


def processArguments(args: argparse.Namespace):
    mapPath: Path = args.mapfile
    expectedMapPath: Path = args.expectedmap
    romPath: Path = args.rompath
    expectedRomPath: Path = args.expectedrom

    diffCount: int = args.count
    mismatchSize: bool = args.mismatch_size

    exit(doFirstDiff(mapPath, expectedMapPath, romPath, expectedRomPath, diffCount, mismatchSize))


def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser]):
    parser = subparser.add_parser("first_diff", help="Find the first difference(s) between the built ROM and the base ROM.")

    parser.add_argument("mapfile", help="Path to a map file", type=Path)
    parser.add_argument("expectedmap", help="Path to the map file in the expected dir", type=Path)
    parser.add_argument("rompath", help="Path to built ROM", type=Path)
    parser.add_argument("expectedrom", help="Path to the expected ROM", type=Path)

    parser.add_argument("-c", "--count", type=int, default=5, help="find up to this many instruction difference(s)")
    parser.add_argument("-m", "--mismatch-size", help="Do not exit early if the ROM sizes does not match", action="store_true")

    parser.set_defaults(func=processArguments)
