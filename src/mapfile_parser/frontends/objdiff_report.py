#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import dataclasses
import decomp_settings
from pathlib import Path
import os

from .. import mapfile
from .. import utils
from ..internals import objdiff_report as report_internal


@dataclasses.dataclass
class SummaryTableConfig:
    doUnits: bool = False
    sort: bool = True
    remaining: bool = False

def doObjdiffReport(
        mapPath: Path,
        outputPath: Path,
        prefixesToTrim: list[str],
        reportCategories: mapfile.ReportCategories,
        *,
        pathIndex: int=2,
        asmPath: Path|None=None,
        nonmatchingsPath: Path|None=None,
        emitCategories: bool=False,
        quiet: bool=False,
        summaryTableConfig: SummaryTableConfig|None=SummaryTableConfig(),
    ) -> int:
    if not mapPath.exists():
        print(f"Could not find mapfile at '{mapPath}'")
        return 1

    mapFile = mapfile.MapFile()
    mapFile.readMapFile(mapPath)

    if emitCategories:
        printDefaultCategories(mapFile, prefixesToTrim)

    mapFile.writeObjdiffReportToFile(
        outputPath,
        prefixesToTrim,
        reportCategories,
        pathIndex=pathIndex,
        asmPath=asmPath,
        nonmatchingsPath=nonmatchingsPath,
    )

    if not quiet and summaryTableConfig is not None and not emitCategories:
        report = report_internal.Report.readFile(outputPath)
        if report is None:
            utils.eprint(f"Unable to read back the generated report at {outputPath}")
            return 1
        table = report.asTableStr(
            do_units=summaryTableConfig.doUnits,
            sort=summaryTableConfig.sort,
            remaining=summaryTableConfig.remaining,
        )
        print(table, end="")

        # Output to GitHub Actions job summary, if available
        summary_path = os.getenv("GITHUB_STEP_SUMMARY")
        if summary_path is not None:
            with open(summary_path, "a", encoding="UTF-8") as summary_file:
                summary_file.write("```\n")
                summary_file.write(table)
                summary_file.write("```\n")

    return 0

def printDefaultCategories(
        mapFile: mapfile.MapFile,
        prefixesToTrim: list[str],
    ):
    if len(prefixesToTrim) == 0:
        # Manage a list of defaults
        prefixesToTrim = [
            "build/lib/",
            "build/src/",
            "build/asm/data/",
            "build/asm/",
            "build/",
        ]

    def removeSuffixes(path: Path) -> Path:
        while path.suffix != "":
            path = path.with_suffix("")
        return path

    def removePrefix(path: Path) -> Path:
        current = str(path)
        # Trim the first prefix found in the list
        for x in prefixesToTrim:
            if current.startswith(x):
                current = current.removeprefix(x)
                break
        current = current.removeprefix("/")
        return Path(current)

    categoriesByPath: list[Category] = []
    categoriesBySegment: list[Category] = []

    def addCategoryPath(categoriesByPath: list[Category], cat_path: str):
        ide = cat_path.strip("/")
        for x in categoriesByPath:
            if x.ide == ide:
                return
        cat = Category(ide, ide, [cat_path])
        categoriesByPath.append(cat)

    def addCategorySegment(categoriesByPath: list[Category], segmentName: str, cat_path: str):
        ide = f"segment_{segmentName}"
        for x in categoriesByPath:
            if x.ide == ide:
                if cat_path not in x.paths:
                    x.paths.append(cat_path)
                return
        cat = Category(ide, f"Segment {segmentName}", [cat_path])
        categoriesByPath.append(cat)

    def reduceCategorySegmentPaths(categoriesByPath: list[Category]):
        """
        We want to reduce the list of paths as much as possible by reducing
        them to their prefixes.
        We want to convert a list of paths like this:
        ```
        - main/audio/sound
        - sys/gtl
        - sys/ml
        - main/sfxlimit
        - main/file
        - sys/vi
        - sys/rdp_reset
        - main/title
        - main/menu
        - sys/om
        - ultralib/src/audio/cspsetbank
        - ultralib/src/audio/cspsetpriority
        - ultralib/src/audio/sndpstop
        - ultralib/src/audio/cseq
        - game
        ```
        To a reduced list like this:
        ```
        - main
        - sys
        - ultralib
        - game
        ```

        But the reduced list must not overlap paths from other segments.
        """

        # Ensure the given shortest path is not present in any other segment
        def isUnique(categoriesByPath: list[Category], shortest: Path, currentIde: str) -> bool:
            for cat2 in categoriesByPath:
                if currentIde == cat2.ide:
                    continue
                for path2 in cat2.paths:
                    if shortest in Path(path2).parents:
                        return False
            return True

        # Build the list of reduced prefixes per segment
        uniques: dict[str, list[Path]] = {}
        for cat in categoriesByPath:
            if len(cat.paths) <= 1:
                continue
            currentList: list[Path] = []
            # Check each path for its reduced value
            for p in cat.paths:
                parents = list(Path(p).parents)
                # Omit the `.` element in parents and iterate from the smaller
                # prefix until the largest.
                for shortest in parents[:-1][::-1]:
                    if shortest in currentList:
                        break
                    if isUnique(categoriesByPath, shortest, cat.ide):
                        currentList.append(shortest)
                        break
            if len(currentList) != 0:
                uniques[cat.ide] = currentList

        # Decide if we want to keep the current path or if we want to replace
        # it with a prefix
        def decide(old: str, prefixes: list[Path]) -> str:
            parents = Path(old).parents
            for prefix in prefixes:
                if prefix in parents:
                    return str(prefix)
            return old

        # Update paths with their prefixes for each segment
        for ide, prefixes in uniques.items():
            for cat in categoriesByPath:
                if cat.ide == ide:
                    newPaths = []
                    for path in cat.paths:
                        p = decide(path, prefixes)
                        # Avoid duplication
                        if p not in newPaths:
                            newPaths.append(p)
                    cat.paths = newPaths
                    break

    realSegmentsSeen = False
    for segment in mapFile:
        if segment.vrom is None and realSegmentsSeen:
            # Usually debug sections
            continue
        if segment.vrom is not None:
            realSegmentsSeen = True
        for section in segment:
            if section.isNoloadSection:
                continue
            suffixless = removeSuffixes(section.filepath)
            prefixless = removePrefix(suffixless)

            parts = prefixless.parts
            if len(parts) > 1:
                # Folder
                cat_path = str(parts[0]) + "/"
            elif len(parts) > 0:
                # Top-level file
                cat_path = str(parts[0])
            else:
                # Huh?
                cat_path = "root"

            addCategoryPath(categoriesByPath, cat_path)
            addCategorySegment(categoriesBySegment, segment.name, str(prefixless))
    reduceCategorySegmentPaths(categoriesBySegment)

    print("""\
tools:
  mapfile_parser:
    progress_report:
      # output: report.json # Optional
      check_asm_paths: True
      # Change if the asm path in the build folder is deeper than two subfolders.
      # i.e.: "build/us/asm/header.o" -> `path_index: 3`.
      # i.e.: "build/us/asm/us/header.o" -> `path_index: 4`.
      # path_index: 2
      prefixes_to_trim:
""", end="")
    for trim in prefixesToTrim:
        print(f"        - {trim}")

    def printCategories(categories: list[Category]):
        for cat in categories:
            ide = cat.ide
            name = cat.name
            if ide[0] in "0123456789":
                ide = f'"{ide}"'
            if name[0] in "0123456789":
                name = f'"{name}"'
            print(f"""\
        - id: {ide}
          name: {name}
          paths:
""", end="")
            for p in cat.paths:
                if p[0] in "0123456789":
                    p = f'"{p}"'
                print(f"""\
            - {p}
""", end="")

    print("      categories:")
    print("        # Categories by path")
    printCategories(categoriesByPath)
    print()
    print("        # Categories by segment")
    printCategories(categoriesBySegment)


def processArguments(args: argparse.Namespace, decompConfig: decomp_settings.Config|None=None):
    reportCategories = mapfile.ReportCategories()
    outputPath: Path
    pathIndexDefault = 2

    settings = SpecificSettings.fromDecompConfig(decompConfig)
    if settings is not None:
        if settings.output is not None:
            outputPath = settings.output
        else:
            outputPath = args.output

        if len(settings.prefixesToTrim) > 0:
            prefixesToTrim = settings.prefixesToTrim
        elif args.prefixes_to_trim is not None:
            prefixesToTrim = list(args.prefixes_to_trim)
        else:
            prefixesToTrim = []

        for cat in settings.categories:
            reportCategories.push(cat.ide, cat.name, cat.paths)

        if settings.pathIndex is not None:
            pathIndex = settings.pathIndex
        elif args.path_index is not None:
            pathIndex = int(args.path_index)
        else:
            pathIndex = pathIndexDefault
    else:
        outputPath = args.output
        if args.prefixes_to_trim is not None:
            prefixesToTrim = list(args.prefixes_to_trim)
        else:
            prefixesToTrim = []
        pathIndex = int(args.path_index) if args.path_index is not None else pathIndexDefault

    if decompConfig is not None:
        version = decompConfig.get_version_by_name(args.version)
        assert version is not None
        mapPath = Path(version.paths.map)

        if version.paths.asm is None:
            asmPath = args.asmpath
        elif settings is not None and settings.checkAsmPaths:
            asmPath = Path(version.paths.asm)
        else:
            asmPath = None

        if version.paths.nonmatchings is None:
            nonmatchingsPath = args.nonmatchingspath
        elif settings is not None and settings.checkAsmPaths:
            nonmatchingsPath = Path(version.paths.nonmatchings)
        else:
            nonmatchingsPath = None
    else:
        mapPath = args.mapfile
        asmPath = args.asmpath
        nonmatchingsPath = args.nonmatchingspath

    emitCategories: bool = args.emit_categories
    if not args.quiet:
        summaryTableConfig = SummaryTableConfig()
    else:
        summaryTableConfig = None

    exit(doObjdiffReport(
        mapPath,
        outputPath,
        prefixesToTrim,
        reportCategories,
        asmPath=asmPath,
        pathIndex=pathIndex,
        nonmatchingsPath=nonmatchingsPath,
        emitCategories=emitCategories,
        summaryTableConfig=summaryTableConfig,
    ))

def addSubparser(subparser: argparse._SubParsersAction[argparse.ArgumentParser], decompConfig: decomp_settings.Config|None=None):
    epilog = """\
Visit https://decomp.dev/ and https://wiki.decomp.dev/tools/decomp-dev for more
information about uploading the generated progress report.

A summary of the generated progress report will be printed to stdout by
default. Also this script will try to detect if it is running on a Github
Action and print the same summary as a step summary. Use the `--quiet` flag to
avoid this behaviour.

This utility has support for a special section on the `decomp.yaml` file, which
allows to avoid passing many arguments to utility.

Use the `--emit-categories` flag to print categories generated automatically
from your mapfile, using the `decomp.yaml` format.
You are expected to tweak the generated categories to accomodate to your liking
instead of using them as-is.
You may also want to set `prefixes_to_trim` before using this flag, to improve
the generation of these categories.

Here's an example for this entry:

```
tools:
  mapfile_parser:
    progress_report:
      # output: report.json # Optional
      check_asm_paths: True
      # path_index: 2
      # List of build prefixes to trim from each object file
      prefixes_to_trim:
        - build/lib/
        - build/src/
        - build/asm/data/
        - build/asm/
        - build/
      # List of categories. `id`s must be unique, but each path may be
      # duplicated across categories.
      categories:
        - id: rom_header
          name: rom_header
          paths:
            - rom_header/

        - id: game_cod
          name: Game code
          paths:
            - main_segment/

        - id: libultra
          name: libultra
          paths:
            - libultra/
            - ultralib/
```
"""

    parser = subparser.add_parser(
        "objdiff_report",
        help="Computes current progress of the matched functions and generates a report suitable for uploading to https://decomp.dev/. Expects `.NON_MATCHING` marker symbols on the mapfile to know which symbols are not matched yet.",
        epilog=epilog,
        formatter_class=argparse.RawTextHelpFormatter,
    )

    emitMapfile = True
    emitOutput = True
    emitAsmpath = True
    emitNonmatchings = True
    emitPrefixesToTrim = True
    emitPathIndex = True
    settings = SpecificSettings.fromDecompConfig(decompConfig)
    if settings is not None:
        assert decompConfig is not None
        versions = []
        for version in decompConfig.versions:
            versions.append(version.name)

        if len(versions) > 0:
            parser.add_argument("-v", "--version", help="Version to process from the decomp.yaml file", type=str, choices=versions, default=versions[0])
            emitMapfile = False
            if decompConfig.versions[0].paths.asm is not None:
                emitAsmpath = False
            if decompConfig.versions[0].paths.nonmatchings is not None:
                emitNonmatchings = False
        if settings.output is not None:
            emitOutput = False
        if len(settings.prefixesToTrim) > 0:
            emitPrefixesToTrim = False
        if settings.pathIndex is not None:
            emitPathIndex = False

    # CLI options exists only if they are not present on the decomp.yaml file
    if emitMapfile:
        parser.add_argument("mapfile", help="Path to a map file.", type=Path)
    if emitOutput:
        parser.add_argument("output", help="Path to output file.", type=Path)

    if emitAsmpath:
        parser.add_argument("-a", "--asmpath", help="Path to asm folder.", type=Path)
    if emitNonmatchings:
        parser.add_argument("-n", "--nonmatchingspath", help="Path to nonmatchings folder.", type=Path)
    if emitPrefixesToTrim:
        parser.add_argument("-t", "--prefixes-to-trim", help="List of path prefixes to try to trim from each object path from the mapfile. For each object they will be tried in order and it will stop at the first prefix found.", action="append")
    if emitPathIndex:
        parser.add_argument("-i", "--path-index", help="Specify the index to start reading the file paths. Defaults to 2", type=int)

    parser.add_argument("--emit-categories", help="Print automatically-generated categories from your mapfile, using the decomp.yaml format. These categories are expected to be tweaked and not used as-is.", action="store_true")
    parser.add_argument("--quiet", help="Avoid printing the progress report to the stdout and to the Github action summary.", action="store_true")

    parser.set_defaults(func=processArguments)


@dataclasses.dataclass
class SpecificSettings:
    output: Path|None
    prefixesToTrim: list[str]
    categories: list[Category]
    pathIndex: int|None
    checkAsmPaths: bool

    @staticmethod
    def fromDecompConfig(decompConfig: decomp_settings.Config|None=None) -> SpecificSettings|None:
        if decompConfig is None:
            return None

        output: Path|None = None
        prefixesToTrim: list[str] = []
        categories: list[Category] = []
        pathIndex: int|None = None
        checkAsmPaths: bool = False
        if decompConfig.tools is not None:
            mapfileParserConfig = decompConfig.tools.get("mapfile_parser")
            if mapfileParserConfig is not None:
                raw = mapfileParserConfig.raw()
                if isinstance(raw, dict):
                    raw = raw.get("progress_report")
                    if isinstance(raw, dict):
                        var = raw.get("output")
                        if var is not None:
                            output = Path(var)
                        var = raw.get("prefixes_to_trim")
                        if var is not None:
                            prefixesToTrim = list(var)
                        var = raw.get("categories")
                        if var is not None:
                            for x in var:
                                cat = Category.from_dict(x)
                                assert cat is not None, f"Invalid category {x}"
                                categories.append(cat)
                        var = raw.get("path_index")
                        if var is not None:
                            pathIndex = var
                        checkAsmPaths = bool(raw.get("check_asm_paths", False))

        return SpecificSettings(
            output,
            prefixesToTrim,
            categories,
            pathIndex,
            checkAsmPaths,
        )

@dataclasses.dataclass
class Category:
    ide: str
    name: str
    paths: list[str]

    @staticmethod
    def from_dict(data: dict|None) -> Category|None:
        if data is None:
            return None

        ide = data.get("id")
        if ide is None:
            return None
        assert isinstance(ide, str), f"{type(ide)} {ide}"

        name = data.get("name")
        if name is None:
            return None
        assert isinstance(name, str), f"{type(name)} {name}"

        paths = data.get("paths")
        if paths is None:
            return None
        assert isinstance(paths, list), f"{type(paths)} {paths}"
        for x in paths:
            assert isinstance(x, str), f"{type(x)} {x}"

        return Category(
            ide,
            name,
            paths,
        )
