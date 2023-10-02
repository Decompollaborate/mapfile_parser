#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from typing import Any
from pathlib import Path


from .mapfile_parser import FoundSymbolInfo as FoundSymbolInfo
from .mapfile_parser import SymbolComparisonInfo as SymbolComparisonInfo
from .mapfile_parser import MapsComparisonInfo as MapsComparisonInfo
from .mapfile_parser import Symbol as Symbol
from .mapfile_parser import File as File
from .mapfile_parser import Segment as Segment
from .mapfile_parser import MapFile as MapFile


def __symbolrs_serializeVram(self: Symbol, humanReadable: bool=True) -> str|int|None:
    if humanReadable:
        return f"0x{self.vram:08X}"
    return self.vram

def __symbolrs_serializeSize(self: Symbol, humanReadable: bool=True) -> str|int|None:
    if self.size is None:
        return None
    if humanReadable:
        return f"0x{self.size:X}"
    return self.size

def __symbolrs_serializeVrom(self: Symbol, humanReadable: bool=True) -> str|int|None:
    if self.vrom is None:
        return None
    if humanReadable:
        return f"0x{self.vrom:06X}"
    return self.vrom

def __symbolrs_toJson(self: Symbol, humanReadable: bool=True) -> dict[str, Any]:
    result: dict[str, Any] = {
        "name": self.name,
        "vram": self.serializeVram(humanReadable=humanReadable),
        "size": self.serializeSize(humanReadable=humanReadable),
        "vrom": self.serializeVrom(humanReadable=humanReadable),
    }
    return result

Symbol.serializeVram = __symbolrs_serializeVram
Symbol.serializeSize = __symbolrs_serializeSize
Symbol.serializeVrom = __symbolrs_serializeVrom
Symbol.toJson = __symbolrs_toJson


@property
def __filers_filepath(self: File) -> Path:
    return Path(self._filepath_internal)
@__filers_filepath.setter
def __filers_filepath(self: File, newPath: Path):
    self._filepath_internal = str(newPath)

def __filers_serializeVram(self: File, humanReadable: bool=True) -> str|int|None:
    if humanReadable:
        return f"0x{self.vram:08X}"
    return self.vram

def __filers_serializeSize(self: File, humanReadable: bool=True) -> str|int|None:
    if humanReadable:
        return f"0x{self.size:X}"
    return self.size

def __filers_serializeVrom(self: File, humanReadable: bool=True) -> str|int|None:
    if self.vrom is None:
        return None
    if humanReadable:
        return f"0x{self.vrom:06X}"
    return self.vrom

def __filers_toJson(self: File, humanReadable: bool=True) -> dict[str, Any]:
    fileDict: dict[str, Any] = {
        "filepath": str(self.filepath),
        "sectionType": self.sectionType,
        "vram": self.serializeVram(humanReadable=humanReadable),
        "size": self.serializeSize(humanReadable=humanReadable),
        "vrom": self.serializeVrom(humanReadable=humanReadable),
    }

    symbolsList = []
    for symbol in self:
        symbolsList.append(symbol.toJson(humanReadable=humanReadable))

    fileDict["symbols"] = symbolsList
    return fileDict

File.filepath = __filers_filepath # type: ignore
File.serializeVram = __filers_serializeVram
File.serializeSize = __filers_serializeSize
File.serializeVrom = __filers_serializeVrom
File.toJson = __filers_toJson


def __segmentrs_serializeVram(self: Segment, humanReadable: bool=True) -> str|int|None:
    if humanReadable:
        return f"0x{self.vram:08X}"
    return self.vram

def __segmentrs_serializeSize(self: Segment, humanReadable: bool=True) -> str|int|None:
    if humanReadable:
        return f"0x{self.size:X}"
    return self.size

def __segmentrs_serializeVrom(self: Segment, humanReadable: bool=True) -> str|int|None:
    if humanReadable:
        return f"0x{self.vrom:06X}"
    return self.vrom

def __segmentrs_toJson(self: Segment, humanReadable: bool=True) -> dict[str, Any]:
    segmentDict: dict[str, Any] = {
        "name": self.name,
        "vram": self.serializeVram(humanReadable=humanReadable),
        "size": self.serializeSize(humanReadable=humanReadable),
        "vrom": self.serializeVrom(humanReadable=humanReadable),
    }

    filesList = []
    for file in self:
        filesList.append(file.toJson(humanReadable=humanReadable))

    segmentDict["files"] = filesList

    return segmentDict

Segment.serializeVram = __segmentrs_serializeVram
Segment.serializeSize = __segmentrs_serializeSize
Segment.serializeVrom = __segmentrs_serializeVrom
Segment.toJson = __segmentrs_toJson



def __mapfilers_printAsCsv(self: MapFile, printVram: bool=True, skipWithoutSymbols: bool=True):
    print(self.toCsv(printVram=printVram, skipWithoutSymbols=skipWithoutSymbols), end="")

def __mapfilers_printSymbolsCsv(self: MapFile):
    print(self.toCsvSymbols(), end="")

def __mapfilers_toJson(self: MapFile, humanReadable: bool=True) -> dict[str, Any]:
    segmentsList = []
    for segment in self:
        segmentsList.append(segment.toJson(humanReadable=humanReadable))

    result: dict[str, Any] = {
        "segments": segmentsList
    }
    return result

MapFile.printAsCsv = __mapfilers_printAsCsv
MapFile.printSymbolsCsv = __mapfilers_printSymbolsCsv
MapFile.toJson = __mapfilers_toJson
