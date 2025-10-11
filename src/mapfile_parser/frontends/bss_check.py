#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023-2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from collections.abc import Callable
import decomp_settings
from pathlib import Path

from .. import mapfile
from .. import utils


def getComparison(
    mapPath: Path,
    expectedMapPath: Path,
    *,
    reverseCheck: bool = True,
    plfResolver: Callable[[Path], Path | None] | None = None,
    plfResolverExpected: Callable[[Path], Path | None] | None = None,
) -> mapfile.MapsComparisonInfo:
    buildMap = mapfile.MapFile.newFromMapFile(mapPath)
    buildMap = buildMap.filterBySectionType(".bss")
    if plfResolver is not None:
        buildMap = buildMap.resolvePartiallyLinkedFiles(plfResolver)

    expectedMap = mapfile.MapFile.newFromMapFile(expectedMapPath)
    expectedMap = expectedMap.filterBySectionType(".bss")
    if plfResolverExpected is not None:
        expectedMap = expectedMap.resolvePartiallyLinkedFiles(plfResolverExpected)

    return buildMap.compareFilesAndSymbols(expectedMap, checkOtherOnSelf=reverseCheck)


def printSymbolComparisonAsCsv(
    comparisonInfo: mapfile.MapsComparisonInfo,
    printAll: bool = False,
    printGoods: bool = True,
):
    print(
        "Symbol Name,Build Address,Build File,Expected Address,Expected File,Difference,GOOD/BAD/MISSING"
    )

    # If it's bad or missing, don't need to do anything special.
    # If it's good, check for if it's in a file with bad or missing stuff, and check if print all is on. If none of these, print it.

    for symbolInfo in comparisonInfo.comparedList:
        buildFile = symbolInfo.buildFile
        expectedFile = symbolInfo.expectedFile
        buildFilePath = buildFile.filepath if buildFile is not None else None
        expectedFilePath = expectedFile.filepath if expectedFile is not None else None

        if symbolInfo.diff is None:
            print(
                f"{symbolInfo.symbol.name},{symbolInfo.buildAddress:X},{buildFilePath},{symbolInfo.expectedAddress:X},{expectedFilePath},{symbolInfo.diff},MISSING"
            )
            continue

        symbolState = "BAD"
        if symbolInfo.diff == 0:
            symbolState = "GOOD"
            if (
                buildFile not in comparisonInfo.badFiles
                and expectedFile not in comparisonInfo.badFiles
            ):
                if (
                    buildFile not in comparisonInfo.badFiles
                    and expectedFile not in comparisonInfo.badFiles
                ):
                    if not printAll:
                        continue

        if not printGoods and symbolState == "GOOD":
            continue

        if buildFile != expectedFile:
            symbolState += " MOVED"
        print(
            f"{symbolInfo.symbol.name},{symbolInfo.buildAddress:X},{buildFilePath},{symbolInfo.expectedAddress:X},{expectedFilePath},{symbolInfo.diff:X},{symbolState}"
        )


def printSymbolComparisonAsListing(
    comparisonInfo: mapfile.MapsComparisonInfo,
    printAll: bool = False,
    printGoods: bool = True,
):
    # print("Symbol Name,Build Address,Build File,Expected Address,Expected File,Difference,GOOD/BAD/MISSING")

    # If it's bad or missing, don't need to do anything special.
    # If it's good, check for if it's in a file with bad or missing stuff, and check if print all is on. If none of these, print it.

    for symbolInfo in comparisonInfo.comparedList:
        buildFile = symbolInfo.buildFile
        expectedFile = symbolInfo.expectedFile
        buildFilePath = buildFile.filepath if buildFile is not None else None
        expectedFilePath = expectedFile.filepath if expectedFile is not None else None

        if symbolInfo.diff is None:
            print(f"Symbol: {symbolInfo.symbol.name} (MISSING)")
            if symbolInfo.buildAddress != -1:
                print(
                    f"    Build:      0x{symbolInfo.buildAddress:08X} (file: {buildFilePath})"
                )
            if symbolInfo.expectedAddress != -1:
                print(
                    f"    Expected:   0x{symbolInfo.expectedAddress:08X} (file: {expectedFilePath})"
                )
            continue

        symbolState = "BAD"
        if symbolInfo.diff == 0:
            symbolState = "GOOD"
            if (
                buildFile not in comparisonInfo.badFiles
                and expectedFile not in comparisonInfo.badFiles
            ):
                if (
                    buildFile not in comparisonInfo.badFiles
                    and expectedFile not in comparisonInfo.badFiles
                ):
                    if not printAll:
                        continue

        if not printGoods and symbolState == "GOOD":
            continue

        if buildFile != expectedFile:
            symbolState += " MOVED"

        if symbolInfo.diff < 0:
            diffStr = f"-0x{-symbolInfo.diff:02X}"
        else:
            diffStr = f"0x{symbolInfo.diff:02X}"

        print(f"Symbol: {symbolInfo.symbol.name} ({symbolState}) (diff: {diffStr})")
        print(
            f"    Build:      0x{symbolInfo.buildAddress:08X} (file: {buildFilePath})"
        )
        print(
            f"    Expected:   0x{symbolInfo.expectedAddress:08X} (file: {expectedFilePath})"
        )


def printSymbolComparison(
    comparisonInfo: mapfile.MapsComparisonInfo,
    printAll: bool = False,
    printGoods: bool = True,
    printingStyle: str = "csv",
):
    if printingStyle == "csv":
        printSymbolComparisonAsCsv(comparisonInfo, printAll, printGoods)
    elif printingStyle == "listing":
        printSymbolComparisonAsListing(comparisonInfo, printAll, printGoods)
    else:
        printSymbolComparisonAsListing(comparisonInfo, printAll, printGoods)


def printFileComparison(comparisonInfo: mapfile.MapsComparisonInfo):
    utils.eprint("")

    if len(comparisonInfo.badFiles) != 0:
        utils.eprint("  BAD")

        for file in comparisonInfo.badFiles:
            utils.eprint(f"bss reordering in {file.filepath}")
        utils.eprint("")

    if len(comparisonInfo.missingFiles) != 0:
        utils.eprint("  MISSING")

        for file in comparisonInfo.missingFiles:
            utils.eprint(f"Symbols missing from {file.filepath}")
        utils.eprint("")

        utils.eprint(
            "Some files appear to be missing symbols. Have they been renamed or declared as static? You may need to remake 'expected'"
        )


def doBssCheck(
    mapPath: Path,
    expectedMapPath: Path,
    *,
    printAll: bool = False,
    reverseCheck: bool = True,
    plfResolver: Callable[[Path], Path | None] | None = None,
    plfResolverExpected: Callable[[Path], Path | None] | None = None,
) -> int:
    if not mapPath.exists():
        utils.eprint(f"{mapPath} must exist")
        return 1
    if not expectedMapPath.exists():
        utils.eprint(f"{expectedMapPath} must exist")
        return 1

    comparisonInfo = getComparison(
        mapPath,
        expectedMapPath,
        reverseCheck=reverseCheck,
        plfResolver=plfResolver,
        plfResolverExpected=plfResolverExpected,
    )
    printSymbolComparison(comparisonInfo, printAll)

    if len(comparisonInfo.badFiles) + len(comparisonInfo.missingFiles) != 0:
        printFileComparison(comparisonInfo)
        return 1

    utils.eprint("")
    utils.eprint("  GOOD")

    return 0


def processArguments(
    args: argparse.Namespace, decompConfig: decomp_settings.Config | None = None
):
    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        assert version is not None, f"Invalid version '{args.version}' selected"

        mapPath = Path(version.paths.map)

        expectedDir = version.paths.expected_dir
        if expectedDir is not None:
            expectedMapPath = expectedDir / mapPath
        else:
            expectedMapPath = args.expectedmap
    else:
        mapPath = args.mapfile
        expectedMapPath = args.expectedmap

    printAll: bool = args.print_all
    reverseCheck: bool = not args.no_reverse_check
    plfExt: list[str] | None = args.plf_ext

    plfResolver = None
    if plfExt is not None:

        def resolver(x: Path) -> Path | None:
            if x.suffix in plfExt:
                newPath = x.with_suffix(".map")
                if newPath.exists():
                    return newPath
            return None

        plfResolver = resolver

    exit(
        doBssCheck(
            mapPath,
            expectedMapPath,
            printAll=printAll,
            reverseCheck=reverseCheck,
            plfResolver=plfResolver,
        )
    )


def addSubparser(
    subparser: argparse._SubParsersAction[argparse.ArgumentParser],
    decompConfig: decomp_settings.Config | None = None,
):
    parser = subparser.add_parser(
        "bss_check", help="Check that globally visible bss has not been reordered."
    )

    emitMapfile = True
    emitExpected = True
    if decompConfig is not None:
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)

        if len(versions) > 0:
            parser.add_argument(
                "-v",
                "--version",
                help="Version to process from the decomp.yaml file",
                type=str,
                choices=versions,
                default=versions[0],
            )
            emitMapfile = False
            if decompConfig.versions[0].paths.expected_dir is not None:
                emitExpected = False

    if emitMapfile:
        parser.add_argument("mapfile", help="Path to a map file.", type=Path)
    if emitExpected:
        parser.add_argument(
            "expectedmap", help="Path to the map file in the expected dir.", type=Path
        )

    parser.add_argument(
        "-a",
        "--print-all",
        help="Print all bss, not just non-matching.",
        action="store_true",
    )
    parser.add_argument(
        "--no-reverse-check",
        help="Disable looking for symbols on the expected map that are missing on the built map file.",
        action="store_true",
    )

    parser.add_argument(
        "-x",
        "--plf-ext",
        help="File extension for partially linked files (plf). Will be used to transform the `plf`s path into a mapfile path by replacing the extension. The extension must contain the leading period. This argument can be passed multiple times.",
        action="append",
    )

    parser.set_defaults(func=processArguments)
