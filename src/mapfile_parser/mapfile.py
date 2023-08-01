#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2023 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import re
from typing import Generator
from pathlib import Path

from .progress_stats import ProgressStats
from . import utils


regex_fileDataEntry = re.compile(r"^\s+(?P<section>\.[^\s]+)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<name>[^\s]+)$")
regex_functionEntry = re.compile(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)$")
regex_label = re.compile(r"^(?P<name>\.?L[0-9A-F]{8})$")
regex_fill = re.compile(r"^\s+(?P<fill>\*[^\s\*]+\*)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s*$")
regex_loadAddress = re.compile(r"(?P<name>([^\s]+)?)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<loadaddress>(load address)?)\s+(?P<vrom>0x[^\s]+)$")


@dataclasses.dataclass
class FoundSymbolInfo:
    file: File
    symbol: Symbol
    offset: int = 0

    def getAsStr(self) -> str:
        return f"'{self.symbol.name}' (VRAM: {self.symbol.getVramStr()}, VROM: {self.symbol.getVromStr()}, SIZE: {self.symbol.getSizeStr()}, {self.file.filepath})"

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
class LoadAddressData:
    vram: int
    size: int
    vrom: int

@dataclasses.dataclass
class Symbol:
    name: str
    vram: int
    size: int = -1 # in bytes
    vrom: int|None = None

    def getVramStr(self) -> str:
        return f"0x{self.vram:08X}"

    def getSizeStr(self) -> str:
        if self.size < 0:
            return "None"
        return f"0x{self.size:X}"

    def getVromStr(self) -> str:
        if self.vrom is None:
            return "None"
        return f"0x{self.vrom:06X}"

    def serializeSize(self) -> str|None:
        if self.size < 0:
            return None
        return f"0x{self.size:X}"


    @staticmethod
    def printCsvHeader():
        print("Symbol name,VRAM,Size in bytes")

    def printAsCsv(self):
        print(f"{self.name},{self.vram:08X},{self.size}")


    def toJson(self) -> dict:
        result = {
            "name": self.name,
            "vram": self.getVramStr(),
            "size": self.serializeSize(),
            "vrom": self.getVromStr(),
        }

        return result


@dataclasses.dataclass
class File:
    filepath: Path
    vram: int
    size: int # in bytes
    sectionType: str
    symbols: list[Symbol] = dataclasses.field(default_factory=list)
    vrom: int|None = None

    @property
    def isNoloadSection(self) -> bool:
        return self.sectionType == ".bss"


    def serializeVram(self) -> str|None:
        return f"0x{self.vram:08X}"

    def serializeSize(self) -> str|None:
        return f"0x{self.size:X}"

    def serializeVrom(self) -> str|None:
        if self.vrom is None:
            return None
        return f"0x{self.vrom:06X}"


    def getName(self) -> Path:
        return Path(*self.filepath.with_suffix("").parts[2:])

    def findSymbolByName(self, symName: str) -> Symbol|None:
        for sym in self.symbols:
            if sym.name == symName:
                return sym
        return None

    def findSymbolByVramOrVrom(self, address: int) -> tuple[Symbol, int]|None:
        prevVram = self.vram
        prevVrom = self.vrom
        prevSym: Symbol|None = None

        isVram = address >= 0x1000000

        for sym in self.symbols:
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
            if (prevSym.vrom is not None and prevSym.vrom + prevSym.size > address) or (isVram and prevSym.vram + prevSym.size > address):
                if isVram:
                    offset = address - prevVram
                else:
                    assert isinstance(prevVrom, int)
                    offset = address - prevVrom
                if offset < 0:
                    return None
                return prevSym, offset

        return None

    @staticmethod
    def printCsvHeader(printVram: bool=True):
        if printVram:
            print("VRAM,", end="")
        print("File,Section type,Num symbols,Max size,Total size,Average size")

    def printAsCsv(self, printVram: bool=True):
        # Calculate stats
        symCount = len(self.symbols)
        maxSize = 0
        averageSize = self.size / (symCount or 1)
        for sym in self.symbols:
            symSize = sym.size
            if symSize > maxSize:
                maxSize = symSize

        if printVram:
            print(f"{self.vram:08X},", end="")
        print(f"{self.filepath},{self.sectionType},{symCount},{maxSize},{self.size},{averageSize:0.2f}")


    def toJson(self) -> dict:
        fileDict: dict = {
            "filepath": str(self.filepath),
            "sectionType": self.sectionType,
            "vram": self.serializeVram(),
            "size": self.serializeSize(),
            "vrom": self.serializeVrom(),
        }

        symbolsList = []
        for symbol in self.symbols:
            symbolsList.append(symbol.toJson())

        fileDict["symbols"] = symbolsList
        return fileDict


    def __iter__(self) -> Generator[Symbol, None, None]:
        for sym in self.symbols:
            yield sym

    def __getitem__(self, index) -> Symbol:
        return self.symbols[index]


@dataclasses.dataclass
class Segment:
    name: str
    vram: int
    size: int
    vrom: int
    _filesList: list[File] = dataclasses.field(default_factory=list)

    def serializeVram(self) -> str|None:
        return f"0x{self.vram:08X}"

    def serializeSize(self) -> str|None:
        return f"0x{self.size:X}"

    def serializeVrom(self) -> str|None:
        return f"0x{self.vrom:06X}"


    def filterBySectionType(self, sectionType: str) -> Segment:
        newSegment = Segment(self.name, self.vram, self.size, self.vrom)

        for file in self._filesList:
            if file.sectionType == sectionType:
                newSegment._filesList.append(file)
        return newSegment

    def getEveryFileExceptSectionType(self, sectionType: str) -> Segment:
        newSegment = Segment(self.name, self.vram, self.size, self.vrom)

        for file in self._filesList:
            if file.sectionType != sectionType:
                newSegment._filesList.append(file)
        return newSegment


    def findSymbolByName(self, symName: str) -> FoundSymbolInfo|None:
        for file in self._filesList:
            sym = file.findSymbolByName(symName)
            if sym is not None:
                return FoundSymbolInfo(file, sym)
        return None

    def findSymbolByVramOrVrom(self, address: int) -> FoundSymbolInfo|None:
        for file in self._filesList:
            pair = file.findSymbolByVramOrVrom(address)
            if pair is not None:
                sym, offset = pair
                return FoundSymbolInfo(file, sym, offset)
        return None


    def mixFolders(self) -> Segment:
        newSegment = Segment(self.name, self.vram, self.size, self.vrom)

        auxDict: dict[Path, list[File]] = dict()

        # Put files in the same folder together
        for file in self._filesList:
            path = Path(*file.getName().parts[:-1])
            if path not in auxDict:
                auxDict[path] = list()
            auxDict[path].append(file)

        # Pretend files in the same folder are one huge file
        for folderPath, filesInFolder in auxDict.items():
            firstFile = filesInFolder[0]

            vram = firstFile.vram
            size = 0
            sectionType = firstFile.sectionType

            symbols = list()
            for file in filesInFolder:
                size += file.size
                for sym in file.symbols:
                    symbols.append(sym)

            newSegment._filesList.append(File(folderPath, vram, size, sectionType, symbols))

        return newSegment

    def printAsCsv(self, printVram: bool=True, skipWithoutSymbols: bool=True):
        for file in self._filesList:
            if skipWithoutSymbols and len(file.symbols) == 0:
                continue

            file.printAsCsv(printVram)
        return

    def printSymbolsCsv(self):
        for file in self._filesList:
            if len(file.symbols) == 0:
                continue

            for sym in file.symbols:
                print(f"{file.filepath},", end="")
                sym.printAsCsv()
        return


    def toJson(self) -> dict:
        segmentDict: dict = {
            "name": self.name,
            "vram": self.serializeVram(),
            "size": self.serializeSize(),
            "vrom": self.serializeVrom(),
        }

        filesList = []
        for file in self._filesList:
            filesList.append(file.toJson())

        segmentDict["files"] = filesList

        return segmentDict


    def __iter__(self) -> Generator[File, None, None]:
        for file in self._filesList:
            yield file

    def __getitem__(self, index) -> File:
        return self._filesList[index]

    def __len__(self) -> int:
        return len(self._filesList)


class MapFile:
    def __init__(self):
        self._segmentsList: list[Segment] = list()
        self.debugging: bool = False

    def readMapFile(self, mapPath: Path):
        tempSegmentsList: list[Segment] = list()
        tempFilesListList: list[list[File]] = list()
        loadAddressData: LoadAddressData|None = None

        with mapPath.open("r") as f:
            mapData = f.read()

            # Skip the stuff we don't care about
            startIndex = 0
            auxVar = 0
            while auxVar != -1:
                startIndex = auxVar
                auxVar = mapData.find("\nLOAD ", startIndex+1)
            auxVar = mapData.find("\n", startIndex+1)
            if auxVar != -1:
                startIndex = auxVar
            mapData = mapData[startIndex:]
        # print(len(mapData))

        inFile = False

        prevLine = ""
        mapLines = mapData.split("\n")
        for line in mapLines:
            if inFile:
                if line.startswith("                "):
                    entryMatch = regex_functionEntry.search(line)

                    # Find function
                    if entryMatch is not None:
                        funcName = entryMatch["name"]
                        funcVram = int(entryMatch["vram"], 16)

                        # Filter out jump table's labels
                        labelMatch = regex_label.search(funcName)
                        if labelMatch is None:
                            tempFilesListList[-1][-1].symbols.append(Symbol(funcName, funcVram))
                        # print(hex(funcVram), funcName)

                else:
                    inFile = False

            if not inFile:
                fillMatch = regex_fill.search(line)
                entryMatch = regex_fileDataEntry.search(line)
                loadAddressMatch = regex_loadAddress.search(line)

                if fillMatch is not None:
                    # Add *fill* size to last file
                    size = int(fillMatch["size"], 16)
                    tempFilesListList[-1][-1].size += size
                elif entryMatch is not None:
                    # Find file
                    filepath = Path(entryMatch["name"])
                    size = int(entryMatch["size"], 16)
                    vram = int(entryMatch["vram"], 16)
                    sectionType = entryMatch["section"]

                    if size > 0:
                        inFile = True
                        tempFile = File(filepath, vram, size, sectionType)
                        if loadAddressData is not None and loadAddressData.vram == vram and not tempFile.isNoloadSection:
                            tempFile.vrom = loadAddressData.vrom
                        tempFilesListList[-1].append(tempFile)

                elif loadAddressMatch is not None:
                    name = loadAddressMatch["name"]
                    vram = int(loadAddressMatch["vram"], 0)
                    size = int(loadAddressMatch["size"], 0)
                    vrom = int(loadAddressMatch["vrom"], 0)

                    if name == "":
                        # If the segment name is too long then this line gets break in two lines
                        name = prevLine

                    loadAddressData = LoadAddressData(vram, size, vrom)
                    tempSegment = Segment(name, vram, size, vrom)
                    tempSegmentsList.append(tempSegment)
                    tempFilesListList.append([])
                    # print(f"'{name}'", type(name), loadAddressData)

            prevLine = line

        vromOffset = 0

        for i, segment in enumerate(tempSegmentsList):
            filesList = tempFilesListList[i]
            for file in filesList:
                acummulatedSize = 0
                symbolsCount = len(file.symbols)

                if file.vrom is not None:
                    vromOffset = file.vrom

                isNoloadSection = file.isNoloadSection
                if not isNoloadSection:
                    file.vrom = vromOffset

                if symbolsCount > 0:
                    symVrom = vromOffset

                    # Calculate size of each symbol
                    for index in range(symbolsCount-1):
                        func = file.symbols[index]
                        nextFunc = file.symbols[index+1]

                        size = (nextFunc.vram - func.vram)
                        acummulatedSize += size

                        file.symbols[index] = Symbol(func.name, func.vram, size)

                        if not isNoloadSection:
                            # Only set vrom of non bss variables
                            file.symbols[index].vrom = symVrom
                            symVrom += size

                    # Calculate size of last symbol of the file
                    func = file.symbols[symbolsCount-1]
                    size = file.size - acummulatedSize
                    file.symbols[symbolsCount-1] = Symbol(func.name, func.vram, size)
                    if not isNoloadSection:
                        file.symbols[symbolsCount-1].vrom = symVrom
                        symVrom += size

                if not isNoloadSection:
                    # Only increment vrom offset for non bss sections
                    vromOffset += file.size

                segment._filesList.append(file)
            self._segmentsList.append(segment)
        return

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

    def findSymbolByVramOrVrom(self, address: int) -> FoundSymbolInfo|None:
        for segment in self._segmentsList:
            info = segment.findSymbolByVramOrVrom(address)
            if info is not None:
                return info
        return None

    def findLowestDifferingSymbol(self, otherMapFile: MapFile) -> tuple[Symbol, File, Symbol|None]|None:
        minVram = None
        found = None
        for builtSegement in self._segmentsList:
            for builtFile in builtSegement:
                for i, builtSym in enumerate(builtFile):
                    expectedSymInfo = otherMapFile.findSymbolByName(builtSym.name)
                    if expectedSymInfo is None:
                        continue

                    expectedSym = expectedSymInfo.symbol
                    if builtSym.vram != expectedSym.vram:
                        if minVram is None or builtSym.vram < minVram:
                            minVram = builtSym.vram
                            prevSym = None
                            if i > 0:
                                prevSym = builtFile[i-1]
                            found = (builtSym, builtFile, prevSym)
        return found


    def mixFolders(self) -> MapFile:
        newMapFile = MapFile()

        newMapFile.debugging = self.debugging

        for segment in self._segmentsList:
            newMapFile._segmentsList.append(segment.mixFolders())

        return newMapFile

    def getProgress(self, asmPath: Path, nonmatchings: Path, aliases: dict[str, str]=dict(), pathIndex: int=2) -> tuple[ProgressStats, dict[str, ProgressStats]]:
        totalStats = ProgressStats()
        progressPerFolder: dict[str, ProgressStats] = dict()

        if self.debugging:
            utils.eprint(f"getProgress():")

        for segment in self._segmentsList:
            for file in segment:
                if len(file.symbols) == 0:
                    continue

                folder = file.filepath.parts[pathIndex]
                if folder in aliases:
                    folder = aliases[folder]

                if folder not in progressPerFolder:
                    progressPerFolder[folder] = ProgressStats()

                if self.debugging:
                    utils.eprint(f"  folder path: {folder}")

                originalFilePath = Path(*file.filepath.parts[pathIndex:])

                extensionlessFilePath = originalFilePath
                while extensionlessFilePath.suffix:
                    extensionlessFilePath = extensionlessFilePath.with_suffix("")

                fullAsmFile = asmPath / extensionlessFilePath.with_suffix(".s")
                wholeFileIsUndecomped = fullAsmFile.exists()

                if self.debugging:
                    utils.eprint(f"  original file path: {originalFilePath}")
                    utils.eprint(f"  extensionless file path: {extensionlessFilePath}")
                    utils.eprint(f"  full asm file: {fullAsmFile}")
                    utils.eprint(f"  whole file is undecomped: {wholeFileIsUndecomped}")

                for func in file.symbols:
                    funcAsmPath = nonmatchings / extensionlessFilePath / f"{func.name}.s"

                    if self.debugging:
                        utils.eprint(f"    Checking function '{funcAsmPath}' (size 0x{func.size:X}) ... ", end="")

                    if wholeFileIsUndecomped:
                        totalStats.undecompedSize += func.size
                        progressPerFolder[folder].undecompedSize += func.size
                        if self.debugging:
                            utils.eprint(f" the whole file is undecomped (no individual function files exist yet)")
                    elif funcAsmPath.exists():
                        totalStats.undecompedSize += func.size
                        progressPerFolder[folder].undecompedSize += func.size
                        if self.debugging:
                            utils.eprint(f" the function hasn't been matched yet (the function file still exists)")
                    else:
                        totalStats.decompedSize += func.size
                        progressPerFolder[folder].decompedSize += func.size
                        if self.debugging:
                            utils.eprint(f" the function is matched! (the function file was not found)")

        return totalStats, progressPerFolder

    def printAsCsv(self, printVram: bool=True, skipWithoutSymbols: bool=True):
        File.printCsvHeader(printVram)
        for segment in self._segmentsList:
            segment.printAsCsv(printVram=printVram, skipWithoutSymbols=skipWithoutSymbols)
        return

    def printSymbolsCsv(self):
        print(f"File,", end="")
        Symbol.printCsvHeader()

        for segment in self._segmentsList:
            segment.printSymbolsCsv()
        return


    def toJson(self) -> dict:
        segmentsList = []
        for segment in self._segmentsList:
            segmentsList.append(segment.toJson())

        result = {
            "segments": segmentsList
        }
        return result


    def __iter__(self) -> Generator[Segment, None, None]:
        for file in self._segmentsList:
            yield file

    def __getitem__(self, index) -> Segment:
        return self._segmentsList[index]

    def __len__(self) -> int:
        return len(self._segmentsList)
