#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2025 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
from pathlib import Path
import json

@dataclasses.dataclass
class Report:
    measures: ReportMeasures
    categories: list[ReportCategory]
    units: list[ReportUnit]

    def asTableStr(
        self,
        *,
        do_units: bool=False,
        sort: bool=False,
        remaining: bool=False,
    ) -> str:
        out = ""

        new_cats: list[tuple[str, ReportMeasures]] = []
        if do_units:
            for unit in self.units:
                if unit.measures.total_code == 0:
                    # Drop the ones we don't care about
                    continue
                new_cats.append((unit.name, unit.measures))
        else:
            for cat in self.categories:
                if cat.measures.total_code == 0:
                    # Drop the ones we don't care about
                    continue
                new_cats.append((cat.name, cat.measures))

        # Calculate the size for the first column
        columnSize = 8
        for (name, _) in new_cats:
            if len(name) > columnSize:
                columnSize = len(name)
        columnSize += 1

        categoryStr = "{0:<{1}}".format('Category', columnSize)
        out += f"{categoryStr}: {'DecompedSize':>12} / {'Total':>8} {'OfCategory':>12}%  ({'OfTotal':>20}%)\n"

        entry_str = self.measures.as_entry_str("all", self.measures, columnSize)
        assert entry_str is not None
        out += f"{entry_str}\n"
        out += "\n"

        if sort:
            new_cats.sort(key=lambda cat: (cat[1].matched_code_percent, cat[1].total_code, cat[0]), reverse=True)

        for (name, measures) in new_cats:
            entry_str = measures.as_entry_str(name, self.measures, columnSize)
            if entry_str is None:
                continue
            out += entry_str
            if remaining and measures.matched_code_percent != 100:
                matched_code = measures.matched_code
                assert matched_code is not None
                assert self.measures.total_code is not None
                assert measures.total_code is not None
                decomped_percentage_total = matched_code / self.measures.total_code * 100
                remainingPercentage = measures.total_code / self.measures.total_code * 100 - decomped_percentage_total
                out += f" {remainingPercentage:>8.4f}%"
            out += "\n"

        return out


    @staticmethod
    def fromDict(info: dict) -> Report | None:
        categories: list[ReportCategory] = []
        units: list[ReportUnit] = []

        m = info.get("measures")
        if m is None:
            return None
        measures = ReportMeasures.fromDict(m)
        if measures is None:
            return None

        for c in info["categories"]:
            cat = ReportCategory.fromDict(c)
            if cat is not None:
                categories.append(cat)
        for u in info["units"]:
            unit = ReportUnit.fromDict(u)
            if unit is not None:
                units.append(unit)

        return Report(
            measures = measures,
            categories = categories,
            units = units,
        )

    @staticmethod
    def readFile(path: Path) -> Report | None:
        if not path.exists():
            return None
        with path.open() as f:
            report = json.load(f, parse_float=float, parse_int=int)
        return Report.fromDict(report)

@dataclasses.dataclass
class ReportCategory:
    id: str
    name: str
    measures: ReportMeasures

    @staticmethod
    def fromDict(info: dict) -> ReportCategory | None:
        measures = ReportMeasures.fromDict(info["measures"])
        if measures is None:
            return None
        return ReportCategory(
            id = info.get("id", ""),
            name = info.get("name", ""),
            measures = measures,
        )

@dataclasses.dataclass
class ReportUnit:
    name: str
    measures: ReportMeasures
    # sections: 
    # metadata: 

    @staticmethod
    def fromDict(info: dict) -> ReportUnit | None:
        measures = ReportMeasures.fromDict(info["measures"])
        if measures is None:
            return None
        return ReportUnit(
            name = info["name"],
            measures = measures,
        )

@dataclasses.dataclass
class ReportMeasures:
    fuzzy_match_percent: float
    total_code: int
    matched_code: int
    matched_code_percent: float
    matched_data_percent: float
    matched_functions_percent: float
    complete_code_percent: float
    complete_data_percent: float
    total_units: int

    @staticmethod
    def fromDict(info: dict) -> ReportMeasures|None:
        total_units = info.get("total_units")
        if total_units is None:
            return None
        return ReportMeasures(
            fuzzy_match_percent = info.get("fuzzy_match_percent", 0.0),
            total_code = int(info.get("total_code", 0)),
            matched_code = int(info.get("matched_code", 0)),
            matched_code_percent = info.get("matched_code_percent", 0.0),
            matched_data_percent = info.get("matched_data_percent", 0.0),
            matched_functions_percent = info.get("matched_functions_percent", 0.0),
            complete_code_percent = info.get("complete_code_percent", 0.0),
            complete_data_percent = info.get("complete_data_percent", 0.0),
            total_units = total_units,
        )

    def as_entry_str(self, name: str, total_measures: ReportMeasures, column_size: int) -> str|None:
        categoryStr = "{0:<{1}}".format(name, column_size)
        matched_code = self.matched_code
        total_code = self.total_code
        matched_code_percent = self.matched_code_percent

        if total_measures.total_code == 0:
            return None

        decomped_percentage_total = matched_code / total_measures.total_code * 100
        total_total = total_code / total_measures.total_code * 100

        return f"{categoryStr}: {matched_code:>12} / {total_code:>8} {matched_code_percent:>12.4f}%  ({decomped_percentage_total:>8.4}% / {total_total:>8.4}%)"
