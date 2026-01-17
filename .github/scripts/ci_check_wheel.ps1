# A Powershell port of `ci_check_wheel.sh`. Refer to that script instead.

# Any change made here should be made in `ci_check_wheel.sh` too.

param (
    [Parameter(Mandatory = $true)]
    [string]$PYTHON_VERSION,

    [Parameter(Mandatory = $true)]
    [string]$KEY,

    [string]$EXTRA
)

# Equivalent to `set -e`
$ErrorActionPreference = "Stop"

if (Test-Path ".venv") {
    Remove-Item -Recurse -Force ".venv"
}

# When $EXTRA is empty powershell passes the argument as an empty argument to
# uv, so we need to explicitly check the argument and only pass it if it is not
# empty to avoid uv from erroring
if ($EXTRA) {
    uv venv --no-project .venv -p $PYTHON_VERSION $EXTRA
} else {
    uv venv --no-project .venv -p $PYTHON_VERSION
}

uv run --no-sync python --version

$WHEEL = $(Get-ChildItem -Path .\dist\ -Recurse -Filter "mapfile_parser-*-abi3-*")
if ([string]::IsNullOrEmpty($WHEEL)) {
    Write-Host "Wheel not found"
    exit 1
}
uv pip install --no-cache --no-config $WHEEL

uv run --no-sync python -c "import mapfile_parser; print(help(mapfile_parser.scan_for_config))"
