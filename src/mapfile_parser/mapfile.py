#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import re
from typing import Any, Generator
from pathlib import Path

from .progress_stats import ProgressStats
from . import utils

from .mapfile_rs import MapFile as MapFileRs
from .mapfile_rs import Segment as SegmentRs
from .mapfile_rs import Section as SectionRs
from .mapfile_rs import Symbol as SymbolRs
from .mapfile_rs import ReportCategories as ReportCategories

regex_fileDataEntry = re.compile(r"^\s+(?P<section>\.[^\s]+)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<name>[^\s]+)$")
regex_functionEntry = re.compile(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)$")
# regex_functionEntry = re.compile(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)((\s*=\s*(?P<expression>.+))?)$")
regex_label = re.compile(r"^(?P<name>\.?L[0-9A-F]{8})$")
regex_fill = re.compile(r"^\s+(?P<fill>\*[^\s\*]+\*)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s*$")
regex_segmentEntry = re.compile(r"(?P<name>([^\s]+)?)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<loadaddress>(load address)?)\s+(?P<vrom>0x[^\s]+)$")


@dataclasses.dataclass
class FoundSymbolInfo:
    section: Section
    symbol: Symbol
    offset: int = 0

    def getAsStr(self) -> str:
        return f"'{self.symbol.name}' (VRAM: {self.symbol.getVramStr()}, VROM: {self.symbol.getVromStr()}, SIZE: {self.symbol.getSizeStr()}, {self.section.filepath})"

    def getAsStrPlusOffset(self, symName: str|None=None) -> str:
        if self.offset != 0:
            if symName is not None:
                message = symName
            else:
                message = f"0x{self.symbol.vram + self.offset:X}"
            message += f" is at 0x{self.offset:X} bytes inside"
        else:
            message = "Symbol"
        return f"{message} {self.getAsStr()}"

@dataclasses.dataclass
class SymbolComparisonInfo:
    symbol: Symbol
    buildAddress: int
    buildFile: Section|None
    expectedAddress: int
    expectedFile: Section|None

    @property
    def diff(self) -> int|None:
        if self.buildAddress < 0:
            return None
        if self.expectedAddress < 0:
            return None

        buildAddress = self.buildAddress
        expectedAddress = self.expectedAddress

        # If both symbols are present in the same section then we do a diff
        # between their offsets into their respectives section.
        # This is done as a way to avoid too much noise in case an earlier section
        # did shift.
        if self.buildFile is not None and self.expectedFile is not None:
            if self.buildFile.filepath == self.expectedFile.filepath:
                buildAddress -= self.buildFile.vram
                expectedAddress -= self.expectedFile.vram

        return buildAddress - expectedAddress


class MapsComparisonInfo:
    def __init__(self):
        self.badFiles: set[Section] = set()
        self.missingFiles: set[Section] = set()
        self.comparedList: list[SymbolComparisonInfo] = []


@dataclasses.dataclass
class Symbol:
    name: str
    vram: int
    size: int = 0 # in bytes
    vrom: int|None = None
    align: int|None = None
    nonmatchingSymExists: bool = False
    """
    `true` if a symbol with the same name, but with a `.NON_MATCHING`
    suffix is found in this symbol's section. `false` otherwise.

    Note the symbol with the actual `.NON_MATCHING` will have this member
    set to `false`.
    """

    def getVramStr(self) -> str:
        return f"0x{self.vram:08X}"

    def getSizeStr(self) -> str:
        if self.size is None:
            return "None"
        return f"0x{self.size:X}"

    def getVromStr(self) -> str:
        if self.vrom is None:
            return "None"
        return f"0x{self.vrom:06X}"

    def serializeVram(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.vram:08X}"
        return self.vram

    def serializeSize(self, humanReadable: bool=True) -> str|int|None:
        if self.size is None:
            return None
        if humanReadable:
            return f"0x{self.size:X}"
        return self.size

    def serializeVrom(self, humanReadable: bool=True) -> str|int|None:
        if self.vrom is None:
            return None
        if humanReadable:
            return f"0x{self.vrom:06X}"
        return self.vrom

    @staticmethod
    def printCsvHeader():
        print(Symbol.toCsvHeader())

    def printAsCsv(self):
        print(self.toCsv())


    @staticmethod
    def toCsvHeader() -> str:
        return "Symbol name,VRAM,Size in bytes"

    def toCsv(self) -> str:
        return f"{self.name},{self.vram:08X},{self.size}"

    def toJson(self, humanReadable: bool=True) -> dict[str, Any]:
        result: dict[str, Any] = {
            "name": self.name,
            "vram": self.serializeVram(humanReadable=humanReadable),
            "size": self.serializeSize(humanReadable=humanReadable),
            "vrom": self.serializeVrom(humanReadable=humanReadable),
        }

        return result


    def clone(self) -> Symbol:
        return Symbol(self.name, self.vram, self.size, self.vrom, self.align)


    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Symbol):
            return False
        return self.name == other.name and self.vram == other.vram

    # https://stackoverflow.com/a/56915493/6292472
    def __hash__(self):
        return hash((self.name, self.vram))


@dataclasses.dataclass
class Section:
    filepath: Path
    vram: int
    size: int # in bytes
    sectionType: str
    vrom: int|None = None
    align: int|None = None
    isFill: bool = False
    _symbols: list[Symbol] = dataclasses.field(default_factory=list)

    @property
    def isNoloadSection(self) -> bool:
        return self.sectionType in {".bss", ".sbss", "COMMON", ".scommon"}


    def serializeVram(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.vram:08X}"
        return self.vram

    def serializeSize(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.size:X}"
        return self.size

    def serializeVrom(self, humanReadable: bool=True) -> str|int|None:
        if self.vrom is None:
            return None
        if humanReadable:
            return f"0x{self.vrom:06X}"
        return self.vrom


    #! @deprecated
    def getName(self) -> Path:
        return Path(*self.filepath.with_suffix("").parts[2:])

    def findSymbolByName(self, symName: str) -> Symbol|None:
        for sym in self._symbols:
            if sym.name == symName:
                return sym
        return None

    #! @deprecated: Use either `findSymbolByVram` or `findSymbolByVrom` instead.
    def findSymbolByVramOrVrom(self, address: int) -> tuple[Symbol, int]|None:
        prevVram = self.vram
        prevVrom = self.vrom
        prevSym: Symbol|None = None

        isVram = address >= 0x1000000

        for sym in self._symbols:
            if sym.vram == address:
                return sym, 0
            if sym.vrom == address:
                return sym, 0

            if prevSym is not None:
                if (sym.vrom is not None and sym.vrom > address) or (isVram and sym.vram > address):
                    if isVram:
                        offset = address - prevVram
                    else:
                        assert isinstance(prevVrom, int)
                        offset = address - prevVrom
                    if offset < 0:
                        return None
                    return prevSym, offset

            prevVram = sym.vram
            prevVrom = sym.vrom
            prevSym = sym

        if prevSym is not None:
            if (prevSym.vrom is not None and prevSym.size is not None and prevSym.vrom + prevSym.size > address) or (isVram and prevSym.size is not None and prevSym.vram + prevSym.size > address):
                if isVram:
                    offset = address - prevVram
                else:
                    assert isinstance(prevVrom, int)
                    offset = address - prevVrom
                if offset < 0:
                    return None
                return prevSym, offset

        return None

    def findSymbolByVram(self, address: int) -> tuple[Symbol, int]|None:
        prevSym: Symbol|None = None

        for sym in self._symbols:
            if sym.vram == address:
                return sym, 0

            if prevSym is not None:
                if sym.vram > address:
                    offset = address - prevSym.vram
                    if offset < 0:
                        return None
                    return prevSym, offset

            prevSym = sym

        if prevSym is not None:
            if prevSym.size is not None and prevSym.vram + prevSym.size > address:
                offset = address - prevSym.vram
                if offset < 0:
                    return None
                return prevSym, offset

        return None

    def findSymbolByVrom(self, address: int) -> tuple[Symbol, int]|None:
        prevVrom = self.vrom if self.vrom is not None else 0
        prevSym: Symbol|None = None

        for sym in self._symbols:
            if sym.vrom == address:
                return sym, 0

            if prevSym is not None:
                if sym.vrom is not None and sym.vrom > address:
                    offset = address - prevVrom
                    if offset < 0:
                        return None
                    return prevSym, offset

            if sym.vrom is not None:
                prevVrom = sym.vrom
            prevSym = sym

        if prevSym is not None:
            if prevSym.vrom is not None and prevSym.size is not None and prevSym.vrom + prevSym.size > address:
                offset = address - prevVrom
                if offset < 0:
                    return None
                return prevSym, offset

        return None


    @staticmethod
    def printCsvHeader(printVram: bool=True):
        print(Section.toCsvHeader(printVram=printVram))

    def printAsCsv(self, printVram: bool=True):
        print(self.toCsv(printVram=printVram))


    @staticmethod
    def toCsvHeader(printVram: bool=True) -> str:
        ret = ""
        if printVram:
            ret += "VRAM,"
        ret += "Section,Section type,Num symbols,Max size,Total size,Average size"
        return ret

    def toCsv(self, printVram: bool=True) -> str:
        # Calculate stats
        symCount = len(self._symbols)
        maxSize = 0
        averageSize = self.size / (symCount or 1)
        for sym in self._symbols:
            if sym.size is not None and sym.size > maxSize:
                maxSize = sym.size

        ret = ""
        if printVram:
            ret += f"{self.vram:08X},"
        ret += f"{self.filepath},{self.sectionType},{symCount},{maxSize},{self.size},{averageSize:0.2f}"
        return ret

    def toJson(self, humanReadable: bool=True) -> dict[str, Any]:
        fileDict: dict[str, Any] = {
            "filepath": str(self.filepath),
            "sectionType": self.sectionType,
            "vram": self.serializeVram(humanReadable=humanReadable),
            "size": self.serializeSize(humanReadable=humanReadable),
            "vrom": self.serializeVrom(humanReadable=humanReadable),
        }

        symbolsList = []
        for symbol in self._symbols:
            symbolsList.append(symbol.toJson(humanReadable=humanReadable))

        fileDict["symbols"] = symbolsList
        return fileDict

    def asStr(self) -> str:
        return f"{self.filepath}({self.sectionType}) (VRAM: {self.serializeVram(True)}, VROM: {self.serializeVrom(True)}, SIZE: {self.serializeSize(humanReadable=True)})"


    def copySymbolList(self) -> list[Symbol]:
        """Returns a copy (not a reference) of the internal symbol list"""
        return list(self._symbols)

    def setSymbolList(self, newList: list[Symbol]) -> None:
        """Replaces the internal symbol list with a copy of `newList`"""
        self._symbols = list(newList)

    def appendSymbol(self, sym: Symbol) -> None:
        """Appends a copy of `sym` into the internal symbol list"""
        self._symbols.append(sym)


    def clone(self) -> Section:
        f = Section(self.filepath, self.vram, self.size, self.sectionType, self.vrom, self.align)
        for sym in self._symbols:
            f._symbols.append(sym.clone())
        return f


    def __iter__(self) -> Generator[Symbol, None, None]:
        for sym in self._symbols:
            yield sym

    def __getitem__(self, index) -> Symbol:
        return self._symbols[index]

    def __setitem__(self, index, sym: Symbol):
        self._symbols[index] = sym

    def __len__(self) -> int:
        return len(self._symbols)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Section):
            return False
        return self.filepath == other.filepath

    # https://stackoverflow.com/a/56915493/6292472
    def __hash__(self):
        return hash((self.filepath,))


@dataclasses.dataclass
class Segment:
    name: str
    vram: int
    size: int
    vrom: int|None
    align: int|None = None
    _sectionsList: list[Section] = dataclasses.field(default_factory=list)

    def serializeVram(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.vram:08X}"
        return self.vram

    def serializeSize(self, humanReadable: bool=True) -> str|int|None:
        if humanReadable:
            return f"0x{self.size:X}"
        return self.size

    def serializeVrom(self, humanReadable: bool=True) -> str|int|None:
        if self.vrom is None:
            return None
        if humanReadable:
            return f"0x{self.vrom:06X}"
        return self.vrom


    def filterBySectionType(self, sectionType: str) -> Segment:
        newSegment = Segment(self.name, self.vram, self.size, self.vrom)

        for section in self._sectionsList:
            if section.sectionType == sectionType:
                newSegment._sectionsList.append(section)
        return newSegment

    def getEverySectionExceptSectionType(self, sectionType: str) -> Segment:
        newSegment = Segment(self.name, self.vram, self.size, self.vrom)

        for section in self._sectionsList:
            if section.sectionType != sectionType:
                newSegment._sectionsList.append(section)
        return newSegment

    #! @deprecated: Use either `getEverySectionExceptSectionType` instead.
    def getEveryFileExceptSectionType(self, sectionType: str) -> Segment:
        return self.getEverySectionExceptSectionType(sectionType)


    def findSymbolByName(self, symName: str) -> FoundSymbolInfo|None:
        for section in self._sectionsList:
            sym = section.findSymbolByName(symName)
            if sym is not None:
                return FoundSymbolInfo(section, sym)
        return None

    #! @deprecated: Use either `findSymbolByVram` or `findSymbolByVrom` instead.
    def findSymbolByVramOrVrom(self, address: int) -> FoundSymbolInfo|None:
        for section in self._sectionsList:
            pair = section.findSymbolByVramOrVrom(address)
            if pair is not None:
                sym, offset = pair
                return FoundSymbolInfo(section, sym, offset)
        return None

    def findSymbolByVram(self, address: int) -> tuple[FoundSymbolInfo|None, list[Section]]:
        possibleFiles: list[Section] = []
        for section in self._sectionsList:
            pair = section.findSymbolByVram(address)
            if pair is not None:
                sym, offset = pair
                return FoundSymbolInfo(section, sym, offset), []
            if address >= section.vram and address < section.vram + section.size:
                possibleFiles.append(section)
        return None, possibleFiles

    def findSymbolByVrom(self, address: int) -> tuple[FoundSymbolInfo|None, list[Section]]:
        possibleFiles: list[Section] = []
        for section in self._sectionsList:
            if section.vrom is None:
                continue
            pair = section.findSymbolByVrom(address)
            if pair is not None:
                sym, offset = pair
                return FoundSymbolInfo(section, sym, offset), []
            if address >= section.vrom and address < section.vrom + section.size:
                possibleFiles.append(section)
        return None, possibleFiles


    def mixFolders(self) -> Segment:
        newSegment = Segment(self.name, self.vram, self.size, self.vrom)

        auxDict: dict[Path, list[Section]] = dict()

        # Put files in the same folder together
        for section in self._sectionsList:
            path = Path(*section.getName().parts[:-1])
            if path not in auxDict:
                auxDict[path] = list()
            auxDict[path].append(section)

        # Pretend files in the same folder are one huge section
        for folderPath, filesInFolder in auxDict.items():
            firstFile = filesInFolder[0]

            vram = firstFile.vram
            size = 0
            vrom = firstFile.vrom
            sectionType = firstFile.sectionType

            symbols = list()
            for section in filesInFolder:
                size += section.size
                for sym in section:
                    symbols.append(sym)

            tempFile = Section(folderPath, vram, size, sectionType, vrom)
            tempFile.setSymbolList(symbols)
            newSegment._sectionsList.append(tempFile)

        return newSegment


    def printAsCsv(self, printVram: bool=True, skipWithoutSymbols: bool=True):
        print(self.toCsv(printVram=printVram, skipWithoutSymbols=skipWithoutSymbols), end="")

    def printSymbolsCsv(self):
        print(self.toCsvSymbols(), end="")


    def toCsv(self, printVram: bool=True, skipWithoutSymbols: bool=True) -> str:
        ret = ""
        for section in self._sectionsList:
            if skipWithoutSymbols and len(section) == 0:
                continue

            ret += section.toCsv(printVram=printVram) + "\n"
        return ret

    def toCsvSymbols(self) -> str:
        ret = ""

        for section in self._sectionsList:
            if len(section) == 0:
                continue

            for sym in section:
                ret += f"{section.filepath},"
                ret += sym.toCsv()
                ret += "\n"
        return ret

    def toJson(self, humanReadable: bool=True) -> dict[str, Any]:
        segmentDict: dict[str, Any] = {
            "name": self.name,
            "vram": self.serializeVram(humanReadable=humanReadable),
            "size": self.serializeSize(humanReadable=humanReadable),
            "vrom": self.serializeVrom(humanReadable=humanReadable),
        }

        filesList = []
        for section in self._sectionsList:
            filesList.append(section.toJson(humanReadable=humanReadable))

        segmentDict["files"] = filesList

        return segmentDict


    def copySectionList(self) -> list[Section]:
        """Returns a copy (not a reference) of the internal section list"""
        return list(self._sectionsList)

    def setSectionList(self, newList: list[Section]) -> None:
        """Replaces the internal section list with a copy of `newList`"""
        self._sectionsList = list(newList)

    def appendSection(self, section: Section) -> None:
        """Appends a copy of `section` into the internal section list"""
        self._sectionsList.append(section)


    #! @deprecated: Use either `copySectionList` instead.
    def copyFileList(self) -> list[Section]:
        """Returns a copy (not a reference) of the internal section list"""
        return self.copySectionList()

    #! @deprecated: Use either `setSectionList` instead.
    def setFileList(self, newList: list[Section]) -> None:
        """Replaces the internal section list with a copy of `newList`"""
        return self.setSectionList(newList)

    #! @deprecated: Use either `appendSection` instead.
    def appendFile(self, section: Section) -> None:
        """Appends a copy of `section` into the internal section list"""
        return self.appendSection(section)


    def clone(self) -> Segment:
        s = Segment(self.name, self.vram, self.size, self.vrom, self.align)
        for f in self._sectionsList:
            s._sectionsList.append(f.clone())
        return s


    def __iter__(self) -> Generator[Section, None, None]:
        for section in self._sectionsList:
            yield section

    def __getitem__(self, index) -> Section:
        return self._sectionsList[index]

    def __len__(self) -> int:
        return len(self._sectionsList)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Segment):
            return False
        return self.name == other.name and self.vram == other.vram and self.size == other.size and self.vrom == other.vrom

    # https://stackoverflow.com/a/56915493/6292472
    def __hash__(self):
        return hash((self.name, self.vram, self.size, self.vrom))


class MapFile:
    #! @deprecated: Use either `newFromMapFile` or `newFromMapStr` instead.
    def __init__(self):
        self._segmentsList: list[Segment] = list()

        #! @deprecated
        self.debugging: bool = False

    @staticmethod
    def newFromMapFile(mapPath: Path) -> MapFile:
        mapfile = MapFile()
        mapfile.readMapFile(mapPath)
        return mapfile

    @staticmethod
    def newFromMapStr(mapContents: str) -> MapFile:
        mapfile = MapFile()
        mapfile.parseMapContents(mapContents)
        return mapfile

    @staticmethod
    def newFromGnuMapStr(mapContents: str) -> MapFile:
        mapfile = MapFile()
        mapfile.parseMapContentsGNU(mapContents)
        return mapfile

    @staticmethod
    def newFromLldMapStr(mapContents: str) -> MapFile:
        mapfile = MapFile()
        mapfile.parseMapContentsLLD(mapContents)
        return mapfile

    @staticmethod
    def newFromMwMapStr(mapContents: str) -> MapFile:
        """
        Parses the contents of a Metrowerks ld (mwld) map.

        The `map_contents` argument must contain the contents of a Metrowerks ld mapfile.
        """

        nativeMapFile = MapFileRs.newFromMwMapStr(mapContents)

        mapfile = MapFile()
        mapfile._transferContentsFromNativeMapFile(nativeMapFile)
        return mapfile

    def _transferContentsFromNativeMapFile(self, nativeMapFile: MapFileRs):
        for segment in nativeMapFile:
            newSegment = Segment(segment.name, segment.vram, segment.size, segment.vrom, segment.align)
            for section in segment:
                newSection = Section(section.filepath, section.vram, section.size, section.sectionType, section.vrom, section.align, section.isFill)
                for symbol in section:
                    newSymbol = Symbol(symbol.name, symbol.vram, symbol.size, symbol.vrom, symbol.align, symbol.nonmatchingSymExists)

                    newSection._symbols.append(newSymbol)
                newSegment._sectionsList.append(newSection)
            self._segmentsList.append(newSegment)

    def _transferContentsToNativeMapFile(self) -> MapFileRs:
        nativeMapFile = MapFileRs()

        for segment in self._segmentsList:
            newSegment = SegmentRs(segment.name, segment.vram, segment.size, segment.vrom, segment.align)
            for section in segment._sectionsList:
                newSection = SectionRs(section.filepath, section.vram, section.size, section.sectionType, section.vrom, section.align, section.isFill)
                for symbol in section._symbols:
                    size = symbol.size if symbol.size is not None else 0
                    newSymbol = SymbolRs(symbol.name, symbol.vram, size, symbol.vrom, symbol.align, symbol.nonmatchingSymExists)

                    newSection.appendSymbol(newSymbol)
                newSegment.appendFile(newSection)
            nativeMapFile.appendSegment(newSegment)

        return nativeMapFile

    #! @deprecated: Use either `newFromMapFile` instead.
    def readMapFile(self, mapPath: Path):
        """
        Opens the mapfile pointed by the `mapPath` argument and parses it.

        The format of the map will be guessed based on its contents.

        Currently supported map formats:
        - GNU ld
        - clang ld.lld
        - Metrowerks ld
        """

        nativeMapFile = MapFileRs()
        nativeMapFile.readMapFile(mapPath)

        self._transferContentsFromNativeMapFile(nativeMapFile)

    #! @deprecated: Use either `newFromMapStr` instead.
    def parseMapContents(self, mapContents: str):
        """
        Parses the contents of the map.

        The `mapContents` argument must contain the contents of a mapfile.

        The format of the map will be guessed based on its contents.

        Currently supported mapfile formats:
        - GNU ld
        - clang ld.lld
        - Metrowerks ld
        """

        nativeMapFile = MapFileRs()
        nativeMapFile.parseMapContents(mapContents)

        self._transferContentsFromNativeMapFile(nativeMapFile)

    #! @deprecated: Use either `newFromGnuMapStr` instead.
    def parseMapContentsGNU(self, mapContents: str):
        """
        Parses the contents of a GNU ld map.

        The `mapContents` argument must contain the contents of a GNU ld mapfile.
        """

        nativeMapFile = MapFileRs()
        nativeMapFile.parseMapContentsGNU(mapContents)

        self._transferContentsFromNativeMapFile(nativeMapFile)

    #! @deprecated: Use either `newFromLldMapStr` instead.
    def parseMapContentsLLD(self, mapContents: str):
        """
        Parses the contents of a clang ld.lld map.

        The `mapContents` argument must contain the contents of a clang ld.lld mapfile.
        """

        nativeMapFile = MapFileRs()
        nativeMapFile.parseMapContentsLLD(mapContents)

        self._transferContentsFromNativeMapFile(nativeMapFile)


    def filterBySectionType(self, sectionType: str) -> MapFile:
        newMapFile = MapFile()

        newMapFile.debugging = self.debugging

        for segment in self._segmentsList:
            newSegment = segment.filterBySectionType(sectionType)
            if len(newSegment) != 0:
                newMapFile._segmentsList.append(newSegment)
        return newMapFile

    def getEveryFileExceptSectionType(self, sectionType: str) -> MapFile:
        newMapFile = MapFile()

        newMapFile.debugging = self.debugging

        for segment in self._segmentsList:
            newSegment = segment.getEveryFileExceptSectionType(sectionType)
            if len(newSegment) != 0:
                newMapFile._segmentsList.append(newSegment)
        return newMapFile


    def findSymbolByName(self, symName: str) -> FoundSymbolInfo|None:
        for segment in self._segmentsList:
            info = segment.findSymbolByName(symName)
            if info is not None:
                return info
        return None

    #! @deprecated: Use either `findSymbolByVram` or `findSymbolByVrom` instead.
    def findSymbolByVramOrVrom(self, address: int) -> FoundSymbolInfo|None:
        for segment in self._segmentsList:
            info = segment.findSymbolByVramOrVrom(address)
            if info is not None:
                return info
        return None

    def findSymbolByVram(self, address: int) -> tuple[FoundSymbolInfo|None, list[Section]]:
        """
        Returns a symbol with the specified VRAM address (or with an addend) if
        it exists on the mapfile.

        If no symbol if found, then a list of possible files where this symbol
        may belong to is returned. This may happen if the symbol is not
        globally visible.
        """

        possibleFiles: list[Section] = []
        for segment in self._segmentsList:
            info, possibleFilesAux = segment.findSymbolByVram(address)
            if info is not None:
                return info, []
            possibleFiles.extend(possibleFilesAux)
        return None, possibleFiles

    def findSymbolByVrom(self, address: int) -> tuple[FoundSymbolInfo|None, list[Section]]:
        """
        Returns a symbol with the specified VRAM address (or with an addend) if
        it exists on the mapfile.

        If no symbol if found, then a list of possible files where this symbol
        may belong to is returned. This may happen if the symbol is not
        globally visible.
        """

        possibleFiles: list[Section] = []
        for segment in self._segmentsList:
            info, possibleFilesAux = segment.findSymbolByVrom(address)
            if info is not None:
                return info, []
            possibleFiles.extend(possibleFilesAux)
        return None, possibleFiles

    def findLowestDifferingSymbol(self, otherMapFile: MapFile) -> tuple[Symbol, Section, Symbol|None]|None:
        minVram = None
        found = None
        foundIndices = (0, 0)
        for i, builtSegement in enumerate(self._segmentsList):
            for j, builtFile in enumerate(builtSegement):
                for k, builtSym in enumerate(builtFile):
                    expectedSymInfo = otherMapFile.findSymbolByName(builtSym.name)
                    if expectedSymInfo is None:
                        continue

                    expectedSym = expectedSymInfo.symbol
                    if builtSym.vram != expectedSym.vram:
                        if minVram is None or builtSym.vram < minVram:
                            minVram = builtSym.vram
                            prevSym = None
                            if k > 0:
                                prevSym = builtFile[k-1]
                            found = (builtSym, builtFile, prevSym)
                            foundIndices = (i, j)

        if found is not None and found[2] is None:
            # Previous symbol was not in the same section of the given
            # section, so we try to backtrack until we find any symbol.

            foundBuiltSym, foundBuiltFile, _ = found
            i, j = foundIndices

            # We want to check the previous section, not the current one,
            # since we already know the current one doesn't have a symbol
            # preceding the one we found.
            j -= 1;

            while i >= 0:
                builtSegment = self[i]
                while j >= 0:
                    builtFile = builtSegment[j]

                    if len(builtFile) > 0:
                        found = (foundBuiltSym, foundBuiltFile, builtFile[-1])
                        i = -1
                        j = -1
                        break
                    j -= 1
                i -= 1
                if i >= 0:
                    j = len(self[i]) - 1

        return found


    def mixFolders(self) -> MapFile:
        newMapFile = MapFile()

        newMapFile.debugging = self.debugging

        for segment in self._segmentsList:
            newMapFile._segmentsList.append(segment.mixFolders())

        return newMapFile

    #! @deprecated. This functionality is perform automatically during parsing now.
    def fixupNonMatchingSymbols(self) -> MapFile:
        return self.clone()

    def getProgress(self, asmPath: Path, nonmatchings: Path, aliases: dict[str, str]=dict(), pathIndex: int=2, checkFunctionFiles: bool=True) -> tuple[ProgressStats, dict[str, ProgressStats]]:
        totalStats = ProgressStats()
        progressPerFolder: dict[str, ProgressStats] = dict()

        if self.debugging:
            utils.eprint(f"getProgress():")

        for segment in self._segmentsList:
            for section in segment:
                if len(section) == 0:
                    continue

                folder = section.filepath.parts[pathIndex]
                if folder in aliases:
                    folder = aliases[folder]

                if folder not in progressPerFolder:
                    progressPerFolder[folder] = ProgressStats()

                if self.debugging:
                    utils.eprint(f"  folder path: {folder}")

                originalFilePath = Path(*section.filepath.parts[pathIndex:])

                extensionlessFilePath = originalFilePath
                while extensionlessFilePath.suffix:
                    extensionlessFilePath = extensionlessFilePath.with_suffix("")

                fullAsmFile = asmPath / extensionlessFilePath.with_suffix(".s")
                wholeFileIsUndecomped = fullAsmFile.exists()

                if self.debugging:
                    utils.eprint(f"  original section path: {originalFilePath}")
                    utils.eprint(f"  extensionless section path: {extensionlessFilePath}")
                    utils.eprint(f"  full asm section: {fullAsmFile}")
                    utils.eprint(f"  whole section is undecomped: {wholeFileIsUndecomped}")

                for func in section:
                    if func.name.endswith(".NON_MATCHING"):
                        continue

                    funcAsmPath = nonmatchings / extensionlessFilePath / f"{func.name}.s"

                    symSize = 0
                    if func.size is not None:
                        symSize = func.size

                    if self.debugging:
                        utils.eprint(f"    Checking function '{funcAsmPath}' (size 0x{symSize:X}) ... ", end="")

                    if wholeFileIsUndecomped:
                        totalStats.undecompedSize += symSize
                        progressPerFolder[folder].undecompedSize += symSize
                        if self.debugging:
                            utils.eprint(f" the whole section is undecomped (no individual function files exist yet)")
                    elif self.findSymbolByName(f"{func.name}.NON_MATCHING") is not None:
                        totalStats.undecompedSize += symSize
                        progressPerFolder[folder].undecompedSize += symSize
                        if self.debugging:
                            utils.eprint(f" the function hasn't been matched yet (there's a `.NON_MATCHING` symbol with the same name)")
                    elif checkFunctionFiles and funcAsmPath.exists():
                        totalStats.undecompedSize += symSize
                        progressPerFolder[folder].undecompedSize += symSize
                        if self.debugging:
                            utils.eprint(f" the function hasn't been matched yet (the function section still exists)")
                    else:
                        totalStats.decompedSize += symSize
                        progressPerFolder[folder].decompedSize += symSize
                        if self.debugging:
                            utils.eprint(f" the function is matched! (the function section was not found)")

        return totalStats, progressPerFolder

    def writeObjdiffReportToFile(self, outpath: Path, prefixesToTrim: list[str], reportCategories: ReportCategories, *, pathIndex: int=2, asmPath: Path|None=None, nonmatchingsPath: Path|None=None):
        nativeMapFile = self._transferContentsToNativeMapFile()
        nativeMapFile.writeObjdiffReportToFile(outpath, prefixesToTrim, reportCategories, pathIndex=pathIndex, asmPath=asmPath, nonmatchingsPath=nonmatchingsPath)

    # Useful for finding bss reorders
    def compareFilesAndSymbols(self, otherMapFile: MapFile, *, checkOtherOnSelf: bool=True) -> MapsComparisonInfo:
        compInfo = MapsComparisonInfo()

        for segment in self:
            for section in segment:
                for symbol in section:
                    foundSymInfo = otherMapFile.findSymbolByName(symbol.name)
                    if foundSymInfo is not None:
                        comp = SymbolComparisonInfo(symbol, symbol.vram, section, foundSymInfo.symbol.vram, foundSymInfo.section)
                        compInfo.comparedList.append(comp)
                        if comp.diff != 0:
                            compInfo.badFiles.add(section)
                    else:
                        compInfo.missingFiles.add(section)
                        compInfo.comparedList.append(SymbolComparisonInfo(symbol, symbol.vram, section, -1, None))

        if checkOtherOnSelf:
            for segment in otherMapFile:
                for section in segment:
                    for symbol in section:
                        foundSymInfo = self.findSymbolByName(symbol.name)
                        if foundSymInfo is None:
                            compInfo.missingFiles.add(section)
                            compInfo.comparedList.append(SymbolComparisonInfo(symbol, -1, None, symbol.vram, section))

        return compInfo


    def printAsCsv(self, printVram: bool=True, skipWithoutSymbols: bool=True):
        print(self.toCsv(printVram=printVram, skipWithoutSymbols=skipWithoutSymbols), end="")

    def printSymbolsCsv(self):
        print(self.toCsvSymbols(), end="")


    def toCsv(self, printVram: bool=True, skipWithoutSymbols: bool=True) -> str:
        ret = Section.toCsvHeader(printVram=printVram) + "\n"
        for segment in self._segmentsList:
            ret += segment.toCsv(printVram=printVram, skipWithoutSymbols=skipWithoutSymbols)
        return ret

    def toCsvSymbols(self) -> str:
        ret = f"Section," + Symbol.toCsvHeader() + "\n"

        for segment in self._segmentsList:
            ret += segment.toCsvSymbols()
        return ret

    def toJson(self, humanReadable: bool=True) -> dict[str, Any]:
        segmentsList = []
        for segment in self._segmentsList:
            segmentsList.append(segment.toJson(humanReadable=humanReadable))

        result: dict[str, Any] = {
            "segments": segmentsList
        }
        return result


    def copySegmentList(self) -> list[Segment]:
        """Returns a copy (not a reference) of the internal segment list"""
        return list(self._segmentsList)

    def setSegmentList(self, newList: list[Segment]) -> None:
        """Replaces the internal segment list with a copy of `newList`"""
        self._segmentsList = list(newList)

    def appendSegment(self, segment: Segment) -> None:
        """Appends a copy of `segment` into the internal segment list"""
        self._segmentsList.append(segment)


    def clone(self) -> MapFile:
        m = MapFile()
        m.debugging = self.debugging
        for s in self._segmentsList:
            m._segmentsList.append(s.clone())
        return m


    def __iter__(self) -> Generator[Segment, None, None]:
        for section in self._segmentsList:
            yield section

    def __getitem__(self, index) -> Segment:
        return self._segmentsList[index]

    def __len__(self) -> int:
        return len(self._segmentsList)
