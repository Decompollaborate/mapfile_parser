# mapfile_parser

[![PyPI - Downloads](https://img.shields.io/pypi/dm/mapfile-parser)](https://pypi.org/project/mapfile-parser/)
[![GitHub License](https://img.shields.io/github/license/Decompollaborate/mapfile_parser)](https://github.com/Decompollaborate/mapfile_parser/releases/latest)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/Decompollaborate/mapfile_parser)](https://github.com/Decompollaborate/mapfile_parser/releases/latest)
[![PyPI](https://img.shields.io/pypi/v/mapfile-parser)](https://pypi.org/project/mapfile-parser/)
[![GitHub contributors](https://img.shields.io/github/contributors/Decompollaborate/mapfile_parser?logo=purple)](https://github.com/Decompollaborate/mapfile_parser/graphs/contributors)

Map file parser library focusing decompilation projects.

## Installing

The recommended way to install is using from the PyPi release, via `pip`:

```bash
pip install mapfile-parser
```

In case you want to mess with the latest development version without wanting to clone the repository, then you could use the following command:

```bash
pip uninstall mapfile-parser
pip install git+https://github.com/Decompollaborate/mapfile_parser.git@develop
```

NOTE: Installing the development version is not recommended. Proceed at your own risk.

## Examples

Various cli examples are provided in the [frontends folder](src/mapfile_parser/frontends). Most of them are re-implementations of already existing tools using this library to show how to use this library and inspire new ideas.

The list can be checked in runtime with `python3 -m mapfile_parser --help`.

Each one of them can be executed with `python3 -m mapfile_parser utilityname`, for example `python3 -m mapfile_parser pj64_syms`.

- `first_diff`: Find the first difference(s) between the built ROM and the base ROM.
- `pj64_syms`: Produce a PJ64 compatible symbol map.
- `progress`: Computes current progress of the matched functions. Relies on a [splat](https://github.com/ethteck/splat) folder structure and matched functions not longer having a file.
- `sym_info`: Display various information about a symbol or address.
- `symbol_sizes_csv`: Produces a csv summarizing the files sizes by parsing a map file.
- `upload_frogress`: Uploads current progress (calculated by the `progress` utility) of the matched functions to [frogress](https://github.com/decompals/frogress).

None of the provided cli utilities are meant to be used directly on a command line, because they need a large number of long parameters to them and every repo has their own quirks which would need them to be adapted. Those have been written mostly to facilitate people to write those utilities in a way which accomodates their own repo.
