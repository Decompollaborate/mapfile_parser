# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.9.3] - 2025-06-01

### Fixed

- Try to infer the rom address of sections and segments even when the mapfile
  does not have it explictly on GNU mapfiles.

## [2.9.2] - 2025-05-28

### Changed

- `objdiff_report`:
  - Simplify emitted entries by avoiding using quotes as much as possible when
    using the `--emit-categories` flag.
  - Avoid printing the summary table when using the `--emit-categories`.
  - Allow customizing the output of the summary table a little bit via the
    Python API.

## [2.9.1] - 2025-05-27

### Added

- Add `--emit-categories` flag to `objdiff_report`.
  - Prints to stdout automatically-generated categories from the mapfile.
    Categories will use the `decomp.yaml` format.
  - Intended to facilitate to integrate this progress reporting method.
  - The generated categories are expected to be modified by the user and not
    used as is.
- Print a summary from the generated progress report in `objdiff_report`.
  - Printed to stdout by default.
  - This will be printed as a summary step if the script detects it is being run
    in a Github Action.

### Changed

- Metroweks ld: Parse alignment column.

### Fixed

- GNU maps:
  - Fix parsing the segment's metadata (name, address, etc) when the
    segment does not have a rom address.
  - Properly drop empty segments from the parsed output.
- Fix parsing if the mapfile contains Carriage Returns characters (`\r`).

## [2.9.0] - 2025-05-25

### Added

- Support emitting progress reports using
  [`objdiff`](https://github.com/encounter/objdiff)'s format.
  - Relies mainly on `.NON_MATCHING`-suffixed marker symbols being present on
    the mapfile for each non-matched symbol.
  - Rust:
    - Disabled by default, it is gated by the `objdiff_report` feature.
    - Use `MapFile::get_objdiff_report()` to generate the report.
    - It is recommended to mix with the `serde` feature to make serialization of
      the report easier.
  - Python:
    - Use `MapFile::writeObjdiffReportToFile()` to write an objdiff report as a
      `json` file.
- Add `objdiff_report` as a new CLI utility to the Python library.
  - Generate a simple `objdiff` progress report from your terminal!
  - This utility supports the `decomp.yaml` format. This format is the preferred
    way of invoking the tool, given the long list of arguments needed for the
    tool to properly work.
- Add support for Metrowerks ld mapfiles.
  - Existing catch-all functions will try to guess if the given mapfile
    correspond to this kind of map by default.
  - New functions:
    - `MapFile::new_from_mw_map_str()` (Rust) and `MapFile::newFromMwMapStr()`
      (Python): Parses specifically a Metrowerks ld mapfile without guessing.

### Changed

- Update `decomp_settings` to version 0.0.9.
- Bump MSRV from `1.65.0` to `1.74.0`.

## [2.8.1] - 2025-05-22

### Fixed

- Fix broken CLI utilities when a decomp.yaml file is not detected.

## [2.8.0] - 2025-05-20

### Added

- `File::symbol_match_state_iter()` function. Returns an iterator over
  `SymbolDecompState`, which allows to know if a symbol is considered decompiled
  or not decompiled yet.
- `Section::is_fill`. `true` if the section is a `*fill*` entry.
- `MapFile::get_every_section_except_section_type`. Provides the same
  functionallity as the old `MapFile::get_every_file_except_section_type`
  function.
- `Segment::get_every_section_except_section_type`. Provides the same
  functionallity as the old `Segment::get_every_file_except_section_type`
  function.
- `MapFile::new_from_map_file`, `MapFile::new_from_map_str`,
  `MapFile::new_from_gnu_map_str` and `MapFile::new_from_lld_map_str`.
- `Symbol::nonmatching_sym_exists`. This will be set to `true` if a symbol with
  the same name but with a `.NON_MATCHING` suffix is found on the same section.
  - The other suffixed symbol (`.NON_MATCHING`) is still retained in the section.
  - The suffixed symbol will have this member set to `false`.
- Add support for the `decomp.yaml` specification from the
  [`decomp_settings`](https://github.com/ethteck/decomp_settings) project on all
  the provided CLI utilities.
  - If a `decomp.yaml` file is detected, then every CLI argument that can be
    inferred from that file will be be considered optional instead.
  - Most CLI utilites will also add a new optional "version" argument to allow
    picking the version to process from the `decomp.yaml` file. It defaults to
    the first listed version.

### Changed

- Change `Symbol.size` to `u64` from `Option<u64>`.
- Rename `File` to `Section`.
  - `File` is still available as an alias to `Section`, but it is recommended to
    use the new name instead.
- Detect `.NON_MATCHING` symbols and fix the size of both the real symbol and
  the `.NON_MATCHING` one during parsing.

### Deprecated

- `File`. Use `Section` instead.
- `MapFile::get_every_file_except_section_type`. Use
  `MapFile::get_every_section_except_section_type` instead.
- `Segment::get_every_file_except_section_type`. Use
  `Segment::get_every_section_except_section_type` instead.
- `MapFile::new`. Use either `MapFile::new_from_map_file` or
  `MapFile::new_from_map_str` instead.
- `MapFile::read_map_file`. Use either `MapFile::new_from_map_file` instead.
- `MapFile::parse_map_contents`. Use either `MapFile::new_from_map_str` instead.
- `MapFile::parse_map_contents_gnu`. Use either `MapFile::new_from_gnu_map_str`
  instead.
- `MapFile::parse_map_contents_lld`. Use either `MapFile::new_from_lld_map_str`
  instead.
- Deprecate `MapFile::fixup_non_matching_symbols` and family. This functionality
  is perform automatically during parsing now.
  - Calling this function is effectively a no-op now.

### Fixed

- Avoid pointless internal copy during the parsing of GNU mapfiles.

## [2.7.5] - 2025-05-08

### Fixed

- Fix Rust release.

## [2.7.4] - 2025-03-19

### Fixed

- Fix size calculation for the last symbol of a section not properly accounting
  if the first symbol of said section is a `static` symbol (meaning it is
  missing from the mapfile).
  - This fix only applies to GNU mapfiles.

## [2.7.3] - 2025-02-09

### Fixed

- Fix symbol's VROM calculation not properly accounting for the first symbol of
  the section being a `static` symbol (missing from the mapfile).
  - This fix only applies to GNU mapfiles.

## [2.7.2] - 2024-12-15

### Added

- Prebuilt binaries for Python 3.13.

### Changed

- Python 3.9 or later is now required.
  - Bump from Python 3.8 to 3.9.
  - Older versions can't be checked on CI anymore, so I prefer to not claim to
    support something that may have broken without anybody noticing.
  - Nothing really changed. Just the CI tools I was using are refusing to use
    any Python version older than this. Sorry if you were affected by this.
- Use newer pyo3 version.
  - From 0.20 to 0.23.
  - Updated to avoid warnings with newer Rust versions.
  - Fix issues introduced by updating pyo3.

## [2.7.1] - 2024-09-25

### Added

- Add `--json` flag to `progress` frontend.
  - Prints the output as json instead of using a human readable format.

### Changed

- Improve lifetime usage and avoid unnecessary clones on Rust side.

## [2.7.0] - 2024-09-24

### Added

- `MapFile.findSymbolByVram` and `MapFile.findSymbolByVrom` methods.
  - Allow to search a symbol given a given address. The address will be treated
    as either a vram address or a vrom address respectively.
- Add `--vram`, `--vrom` and `--name` arguments to `sym_info` frontend.
  - Allow to tell to `sym_info` exactly how to treat the argument instead of
    trying to guess how to use it.
- `sym_info` can now detect that an address may belong to a file even when the
  symbol itself may not exist on the mapfile.
  - This can happen for local symbols, for example for rodata literals.

### Deprecated

- `MapFile.findSymbolByVramOrVrom`.
  - Use `MapFile.findSymbolByVram` and `MapFile.findSymbolByVrom` instead.

### Fixed

- Fix typo that prevented using `jsonify`.

## [2.6.0] - 2024-08-26

### Added

- Add new parameters to `bss_check.printSymbolComparison`.
  - `printGoods`: Allows toggling printing the GOOD symbols.
  - `printingStyle`: The style to use to print the symbols, either `"csv"` or
    `"listing"`.
  - TODO: port to Rust.
- New `MapFile.fixupNonMatchingSymbols` method.
  - Allows to fixup size calculation of symbols messed up by symbols with the
    same name suffixed with `.NON_MATCHING`.
- Add support for `.NON_MATCHING` suffix on symbols on progress calculation.
  - If a symbol exists and has a `.NON_MATCHING`-suffixed counterpart then
    consider it not mateched yet.

### Changed

- Minor cleanups.

## [2.5.1] - 2024-08-09

### Fixed

- Fix Github Action file.

## [2.5.0] - 2024-08-09

### Added

- Add Minimal Supported Rust Version (MSRV) to Cargo.toml.
- Add `MapFile::new_from_map_file` function to simplify `MapFile` creation.
- Add `serde` feature to the Rust crate.
  - Allows serializing and deserializing a `MapFile` object using serde.

### Changed

- Tweak symbol comparison logic a bit.
  - Symbol shifting (due to different sizes or extra/missing symbols) should
    not affect comparing non shifted files.
- `Cargo.lock` file is now committed to the repo.
- Change Rust functions to properly take references instead of consuming the
  argument.

### Fixed

- Fix `MapFile::find_lowest_differing_symbol` not returning a previous symbol
  from a previous file if the symbol found is the first symbol from the file.

## [2.4.0] - 2024-03-25

### Added

- Add `endian` argument to `doFirstDiff`.
- Add `--endian` option to `first_diff` script.

### Removed

- Dropped Python 3.7 support.
  - Python 3.8 is the minimum supported version now.

## [2.3.7] - 2024-02-27

### Fixed

- Fix not recognizing file entries which are splited in two lines because its
  section name was too long to fit.

## [2.3.6] - 2024-02-23

### Added

- Add issue templates for bug reports and feature suggestions.

### Fixed

- Fix not recognizing sections that don't start with dots (`.`).

## [2.3.5] - 2024-02-04

### Fixed

- Fix `MapFile.compareFilesAndSymbols` reporting the wrong address as the
  expected address.

## [2.3.4] - 2024-01-26

### Changed

- Frontend scripts now give a better error if the mapfile does not exist.

## [2.3.2] - 2024-01-17

### Added

- Add optional `categoryColumnSize` parameter to `ProgressStats.getHeaderAsStr`
  and `ProgressStats.getEntryAsStr`.
  - Allows to set the size of the first column.

### Fixed

- Fix Rust's implementation of `File` not returning a `pathlib.Path` object for
  the `filepath` member.

## [2.3.1] - 2023-12-23

### Added

- Add a few utility methods to `ProgressStats`.

### Changed

- `pyo3` is no longer needed to use this crate as Rust-only library.
- Updated Rust dependencies.

## [2.3.0] - 2023-11-05

### Added

- Support for parsing clang lld's map fles.
- New functions:
  - `MapFile.parseMapContents`/`MapFile::parse_map_contents`
    - Parses the map contents passed as the argument, without requiring the map
      being on an actual file.
    - The map format will be guessed on the contents. Currently both the GNU ld
      and clang ld.lld map formats are recognized.
  - `MapFile.parseMapContentsGNU`/`MapFile::parse_map_contents_gnu`
    - Parses the map contents passed as the argument, without requiring the map
      being on an actual file.
    - This function only parses the GNU ld map format.
  - `MapFile.parseMapContentsLLD`/`MapFile::parse_map_contents_lld`
    - Parses the map contents passed as the argument, without requiring the map
      being on an actual file.
    - This function only parses the clang ld.lld map format.
- New members:
  - `Symbol.align`/`Symbol::align`, `File.align`/`File::align` and
    `Segment.align`/`Segment::align`: The alignment the given type. This member
    will be filled by the parser only if the mapfile provides this information.

### Changed

- `MapFile.readMapFile`/`MapFile::read_map_file` can now guess the map format
  between any of the known formats.
- Some known symbol names will be automatically filtered out during the parsing
  step.
  - Currently only `gcc2_compiled.` is filtered out.

### Fixed

- Fix parser not detecting `*fill*` lines on GNU ld maps if they specified the
  value that was used for filling/padding.
- `.sbss`, `COMMON` and `.scommon` sections are now properly considered noload
  sections.

## [2.2.1] - 2023-10-08

### Fixed

- Fix Rust crate size being too big
  - crates.io was rejecting the package because of the size
  - Cargo was packaging all the map files and test cases, making the package be
    15 MiB. Now it is around 16.3 KiB

## [2.2.0] - 2023-10-08

### Added

- Add new Rust re-implementation (#15)
  - Allows using this library in native Rust projects
  - It does not replace the Python implementation due to restrictions on how
    Rust bindings work.
  - Python bindings for the Rust implementation exists, but they are not used
    or exposed to the user
- Now this library has a Rust crate.
  - Check it at <https://crates.io/crates/mapfile_parser>

### Changed

- Speed-up the actual parsing of a mapfile by using the native Rust implementation
  - Up to 10 times faster parsing has been registered
- Change build system from `hatchling` to `maturin`

## [2.1.5] - 2023-10-02

### Added

- Add `CHANGELOG.md`
- Add markdown linter to CI

### Changed

- Cleanup the `README.md` a bit
- Update Github Action's dependencies

### Deprecated

- Deprecate `File.getName`
  - The method itself doesn't make sense, instead operate on `File.filepath` directly
- Deprecate `MapFile.debugging`
- Deprecate `progress` frontend's `--debugging` flag

## [2.1.4] - 2023-09-11

### Fixed

- Fix vrom calculation if the first symbol of a file is not available in the
  map file

## [1.3.2] - 2023-09-11

### Added

- Add version info to the cli
- Add small testing suite
- Add machine-friendly/non-human-readable option for json generation.
  - Numbers are outputted as real numbers instead of prettified strings

### Changed

- Allow csv conversion to be written to a file instead of only printing to stdout
- Output `none` instead of `"None"` for symbols with no vrom when generating
  json output.

### Fixed

- Fix vrom calculation if the first symbol of a file is not available in the
  map file

Full changes: <https://github.com/Decompollaborate/mapfile_parser/compare/702a73452059ce4e97cda011e09dc4ef2a7b9dec...ba444b0bbfdfad7fb07347bf656b7fd4381596fb>

## [2.1.3] - 2023-08-30

### Fixed

- Fix version number
  - pypi thought previous version was a prerelease instead of a full release

## [2.1.2] - 2023-08-30

### Added

- Add machine-friendly/non-human-readable option for json generation.
  - Numbers are outputted as real numbers instead of prettified strings
- Add some CI tests

### Changed

- Output `none` instead of `"None"` for symbols with no vrom when generating
  json output.
- Make dummy files for `*fill*` lines instead of adding them to the previous file.
- Don't drop the dummy segment if it actually has data on it

## [2.1.1] - 2023-08-15

### Fixed

- Fix off-by-one issue which was throwing away tons of data

## [2.1.0] - 2023-08-14

### Added

- Add `bss_check` frontend
  - Allows to search for reordered bss variables by comparing two map files.

### Changed

- Don't skip important lines in some kinds of map files.
  - This may produce map parsing to be a bit slower but it should work properly
    with more kinds of mapfiles

### Fixed

- Try to prevent crashes if a file is found before the first segment is found

## [2.0.1] - 2023-08-07

### Changed

- Makes `Symbol`, `File` and `Segment` hashable

## [1.3.1] - 2023-08-06

### Changed

- Make `Symbol` and `File` types hashable

## [2.0.0] - 2023-08-01

### Added

- `toJson` method which allow serializing map files into the json format.
- `jsonify` frontend which allows converting a mapfile into a json format from
  the CLI.

### Changed

- Change logic of `MapFile` so it can parse and organize each file in proper
  segments.
  - This breaks old ways of iterating the `MapFile` class. Now iterating it
    yields a `Segment`, iterating that yields a `File`.
- Rename `segmentType` to `sectionType` and `filterBySegmentType` to
  `filterBySectionType`.

## [1.3.0] - 2023-08-01

### Added

- Add function `toJson` to export map file as a json. It returns a `dict`.
- New `jsonify` frontend, which allows converting a map file in the cli.

## [1.2.1] - 2023-07-28

### Fixed

- Fix missing `:` colon even when passing `addColons=True` to the `first_diff`
  frontend

## [1.2.0] - 2023-07-28

### Added

- `first_diff` frontend:
  - Allow an optional bytes converter callback. It can be useful to perform
    analysis or instruction decoding on the caller side.
  - Parameter to toggle colons (`:`) in bytes difference output

## [1.1.5] - 2023-07-28

### Fixed

- Fix map parsing ignoring some `*fill*` entries
- Improve symbol info output a bit

## [1.1.4] - 2023-04-03

### Fixed

- Add missing `request` requirement

## [1.1.3] - 2023-02-22

### Added

- Add flag to enable debug prints
- Add flag to specify the path index

### Fixed

- Properly handle files with multiple extensions

## [1.1.2] - 2022-12-14

### Changed

- Modularize `upload_frogress` frontend

## [1.1.1] - 2022-12-14

### Added

- Add examples to README

## [1.1.0] - 2022-12-14

### Added

- Compute vrom addresses of symbols and files
- Provide various front-ends clis (see README for more info):
  - `first_diff`
  - `pj64_syms`
  - `progress`
  - `sym_info`
  - `symbol_sizes_csv`
  - `upload_frogress`

[Full changelog](https://github.com/Decompollaborate/mapfile_parser/compare/1.0.0...1.1.0)

## [1.0.0] - 2022-12-13

### Added

- Initial release

[unreleased]: https://github.com/Decompollaborate/mapfile_parser/compare/master...develop
[2.9.3]: https://github.com/Decompollaborate/mapfile_parser/compare/2.9.2...2.9.3
[2.9.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.9.1...2.9.2
[2.9.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.9.0...2.9.1
[2.9.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.8.1...2.9.0
[2.8.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.8.0...2.8.1
[2.8.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.7.5...2.8.0
[2.7.5]: https://github.com/Decompollaborate/mapfile_parser/compare/2.7.4...2.7.5
[2.7.4]: https://github.com/Decompollaborate/mapfile_parser/compare/2.7.3...2.7.4
[2.7.3]: https://github.com/Decompollaborate/mapfile_parser/compare/2.7.2...2.7.3
[2.7.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.7.1...2.7.2
[2.7.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.7.0...2.7.1
[2.7.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.6.0...2.7.0
[2.6.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.5.1...2.6.0
[2.5.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.5.0...2.5.1
[2.5.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.4.0...2.5.0
[2.4.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.7...2.4.0
[2.3.7]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.6...2.3.7
[2.3.6]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.5...2.3.6
[2.3.5]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.4...2.3.5
[2.3.4]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.2...2.3.4
[2.3.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.1...2.3.2
[2.3.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.3.0...2.3.1
[2.3.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.2.1...2.3.0
[2.2.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.2.0...2.2.1
[2.2.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.5...2.2.0
[2.1.5]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.4...2.1.5
[2.1.4]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.3...2.1.4
[1.3.2]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.1...1.3.2
[2.1.3]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.2...2.1.3
[2.1.2]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.1...2.1.2
[2.1.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.1.0...2.1.1
[2.1.0]: https://github.com/Decompollaborate/mapfile_parser/compare/2.0.1...2.1.0
[2.0.1]: https://github.com/Decompollaborate/mapfile_parser/compare/2.0.0...2.0.1
[1.3.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.0...1.3.1
[2.0.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.3.0...2.0.0
[1.3.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.2.1...1.3.0
[1.2.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.2.0...1.2.1
[1.2.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.5...1.2.0
[1.1.5]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.4...1.1.5
[1.1.4]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.3...1.1.4
[1.1.3]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.2...1.1.3
[1.1.2]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.1...1.1.2
[1.1.1]: https://github.com/Decompollaborate/mapfile_parser/compare/1.1.0...1.1.1
[1.1.0]: https://github.com/Decompollaborate/mapfile_parser/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/Decompollaborate/mapfile_parser/releases/tag/1.0.0
