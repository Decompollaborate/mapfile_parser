#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import re
import collections
from pathlib import Path


regex_fileDataEntry = re.compile(r"^\s+(?P<section>[^\s]+)\s+(?P<vram>0x[^\s]+)\s+(?P<size>0x[^\s]+)\s+(?P<name>[^\s]+)$")
regex_functionEntry = re.compile(r"^\s+(?P<vram>0x[^\s]+)\s+(?P<name>[^\s]+)$")
regex_label = re.compile(r"^(?P<name>\.?L[0-9A-F]{8})$")
regex_segmentType = re.compile(r"^ (?P<type>\.[^ ]+) ")

@dataclasses.dataclass
class Function:
    name: str
    vram: int
    size: int # in words

@dataclasses.dataclass
class File:
    name: str
    vram: int
    size: int # in words
    segmentType: str
    functions: list[Function] = dataclasses.field(default_factory=list)

class MapFile:
    def __init__(self, mapPath: Path):
        tempFilesList: list[File] = list()
        self.filesList: list[File] = list()

        with mapPath.open() as f:
            mapData = f.read()

            # Skip the stuff we don't care about
            startIndex = 0
            auxVar = 0
            while auxVar != -1:
                startIndex = auxVar
                auxVar = mapData.find("\nLOAD ", startIndex+1)
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
                            tempFilesList[-1].functions.append(Function(funcName, funcVram, -1))
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
                        name = "/".join(entryMatch["name"].split("/")[2:])
                        name = ".".join(name.split(".")[:-1])
                        size = int(entryMatch["size"], 16) // 4
                        vram = int(entryMatch["vram"], 16)
                        segmentType = typeMatch["type"]

                        if size > 0:
                            inFile = True
                            tempFilesList.append(File(name, vram, size, segmentType))

        for file in tempFilesList:
            acummulatedSize = 0
            funcCount = len(file.functions)

            # Filter out files with no functions
            if funcCount == 0:
                continue

            # Calculate size of each function
            for index in range(funcCount-1):
                func = file.functions[index]
                nextFunc = file.functions[index+1]

                size = (nextFunc.vram - func.vram) // 4
                acummulatedSize += size

                file.functions[index] = Function(func.name, func.vram, size)

            # Calculate size of last function of the file
            func = file.functions[funcCount-1]
            size = file.size - acummulatedSize
            file.functions[funcCount-1] = Function(func.name, func.vram, size)

            self.filesList.append(file)



    def removeDottedSymbols(self):
        resultFileList: list[File] = list()

        for file in self.filesList:
            newFile = File(file.name, file.vram, file.size, file.segmentType)

            lastFunc: Function|None = None

            for func in file.functions:
                newFunc = Function(func.name, func.vram, func.size)

                if func.name.startswith("."):
                    assert lastFunc is not None, file
                    lastFunc.size += func.size
                    continue

                lastFunc = newFunc
                newFile.functions.append(newFunc)

            resultFileList.append(newFile)

        self.filesList = resultFileList


def mixFolders(filesList: list[File]) -> list[File]:
    newFileList: list[File] = list()

    auxDict = collections.OrderedDict()

    # Put files in the same folder together
    for file in filesList:
        path = "/".join(file.name.split("/")[:-1])
        if path not in auxDict:
            auxDict[path] = list()
        auxDict[path].append(file)

    # Pretend files in the same folder are one huge file
    for folderPath in auxDict:
        filesInFolder = auxDict[folderPath]
        firstFile = filesInFolder[0]

        vram = firstFile.vram
        size = 0

        functions = list()
        for file in filesInFolder:
            size += file.size
            for func in file.functions:
                functions.append(func)

        newFileList.append(File(folderPath, vram, size, "TODO", functions))

    return newFileList


def printCsv(filesList: list[File], printVram: bool = True):
    if printVram:
        print("VRAM,", end="")
    print("File,Num functions,Max size,Total size,Average size")

    for file in filesList:
        name = file.name
        vram = file.vram
        size = file.size
        funcCount = len(file.functions)

        if funcCount == 0:
            continue

        # Calculate stats
        maxSize = 0
        averageSize = size/funcCount
        for func in file.functions:
            funcSize = func.size
            if funcSize > maxSize:
                maxSize = funcSize

        if printVram:
            print(f"{vram:08X},", end="")
        print(f"{name},{funcCount},{maxSize},{size},{averageSize:0.2f}")
    return

def printFunctionsCsv(filesList: list[File]):
    print("File,Function name,VRAM,Size in words")

    for file in filesList:
        name = file.name
        funcCount = len(file.functions)

        if funcCount == 0:
            continue

        for func in file.functions:
            print(f"{name},{func.name},{func.vram:08X},{func.size}")
    return
