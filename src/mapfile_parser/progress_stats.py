#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses


@dataclasses.dataclass
class ProgressStats:
    undecompedSize: int = 0
    decompedSize: int = 0

    @property
    def total(self) -> int:
        return self.undecompedSize + self.decompedSize

    def getAsFrogressEntry(self, name: str) -> dict[str, int]:
        categories: dict[str, int] = {}
        categories[name] = self.decompedSize
        categories[f"{name}/total"] = self.total
        return categories

    @staticmethod
    def printHeader():
        print(f"{'Category':<28}: {'DecompedSize':>12} / {'Total':>8} {'OfFolder':>10}%  ({'OfTotal':>20}%)")

    def print(self, category: str, totalStats: ProgressStats):
        print(f"{category:<28}: {self.decompedSize:>12} / {self.total:>8} {self.decompedSize / self.total * 100:>10.4f}%  ({self.decompedSize / totalStats.total * 100:>8.4f}% / {self.total / totalStats.total * 100:>8.4f}%)")


def printStats(totalStats: ProgressStats, progressPerFolder: dict[str, ProgressStats]):
    ProgressStats.printHeader()
    totalStats.print("all", totalStats)
    print()

    for folder, statsEntry in progressPerFolder.items():
        statsEntry.print(folder, totalStats)

