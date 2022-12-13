#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import re
from pathlib import Path


regex_fileDataEntry = re.compile(r"^\s+(?P<section>[^\s]+)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<name>[^\s]+)$")
regex_functionEntry = re.compile(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)$")
regex_label = re.compile(r"^(?P<name>\.?L[0-9A-F]{8})$")
regex_segmentType = re.compile(r"^ (?P<type>\.[^ ]+) ")

@dataclasses.dataclass
class Symbol:
    name: str
    vram: int
    size: int = -1 # in bytes

    @staticmethod
    def printCsvHeader():
        print("Symbol name,VRAM,Size in bytes")

    def printAsCsv(self):
        print(f"{self.name},{self.vram:08X},{self.size}")

@dataclasses.dataclass
class File:
    filepath: Path
    vram: int
    size: int # in bytes
    segmentType: str
    symbols: list[Symbol] = dataclasses.field(default_factory=list)

    def getName(self) -> Path:
        return Path(*self.filepath.with_suffix("").parts[2:])

    @staticmethod
    def printCsvHeader(printVram: bool=True):
        if printVram:
            print("VRAM,", end="")
        print("File,Segment type,Num symbols,Max size,Total size,Average size")

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
        print(f"{self.filepath},{self.segmentType},{symCount},{maxSize},{self.size},{averageSize:0.2f}")


class MapFile:
    def __init__(self):
        self.filesList: list[File] = list()

    def readMapFile(self, mapPath: Path):
        tempFilesList: list[File] = list()

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
                            tempFilesList[-1].symbols.append(Symbol(funcName, funcVram))
                        # print(hex(funcVram), funcName)

                else:
                    inFile = False
            else:
                typeMatch = regex_segmentType.search(line)
                if typeMatch is not None:
                    inFile = False
                    entryMatch = regex_fileDataEntry.search(line)

                    # Find file
                    if entryMatch is not None:
                        filepath = Path(entryMatch["name"])
                        size = int(entryMatch["size"], 16)
                        vram = int(entryMatch["vram"], 16)
                        segmentType = typeMatch["type"]

                        if size > 0:
                            inFile = True
                            tempFilesList.append(File(filepath, vram, size, segmentType))

        for file in tempFilesList:
            acummulatedSize = 0
            funcCount = len(file.symbols)

            if funcCount > 0:
                # Calculate size of each symbol
                for index in range(funcCount-1):
                    func = file.symbols[index]
                    nextFunc = file.symbols[index+1]

                    size = (nextFunc.vram - func.vram)
                    acummulatedSize += size

                    file.symbols[index] = Symbol(func.name, func.vram, size)

                # Calculate size of last symbol of the file
                func = file.symbols[funcCount-1]
                size = file.size - acummulatedSize
                file.symbols[funcCount-1] = Symbol(func.name, func.vram, size)

            self.filesList.append(file)
        return

    def filterBySegmentType(self, segmentType: str) -> MapFile:
        newMapFile = MapFile()

        for file in self.filesList:
            if file.segmentType == segmentType:
                newMapFile.filesList.append(file)
        return newMapFile

    def getEveryFileExceptSegmentType(self, segmentType: str) -> MapFile:
        newMapFile = MapFile()

        for file in self.filesList:
            if file.segmentType != segmentType:
                newMapFile.filesList.append(file)
        return newMapFile


    def mixFolders(self) -> MapFile:
        newMapFile = MapFile()

        auxDict: dict[Path, list[File]] = dict()

        # Put files in the same folder together
        for file in self.filesList:
            path = Path(*file.getName().parts[:-1])
            if path not in auxDict:
                auxDict[path] = list()
            auxDict[path].append(file)

        # Pretend files in the same folder are one huge file
        for folderPath, filesInFolder in auxDict.items():
            firstFile = filesInFolder[0]

            vram = firstFile.vram
            size = 0
            segmentType = firstFile.segmentType

            symbols = list()
            for file in filesInFolder:
                size += file.size
                for sym in file.symbols:
                    symbols.append(sym)

            newMapFile.filesList.append(File(folderPath, vram, size, segmentType, symbols))

        return newMapFile


    def printAsCsv(self, printVram: bool=True, skipWithoutSymbols: bool=True):
        File.printCsvHeader(printVram)
        for file in self.filesList:
            if skipWithoutSymbols and len(file.symbols) == 0:
                continue

            file.printAsCsv(printVram)
        return

    def printSymbolsCsv(self):
        print(f"File,", end="")
        Symbol.printCsvHeader()

        for file in self.filesList:
            if len(file.symbols) == 0:
                continue

            for sym in file.symbols:
                print(f"{file.filepath},", end="")
                sym.printAsCsv()
        return
