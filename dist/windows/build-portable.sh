#!/bin/bash
# Portable package creator for Grafeno Vetor
set -e
cd "$(dirname "$0")/../.."

SRC="target/debug"
DST="dist/windows/GrafenoVetor"

mkdir -p "$DST"
cp "$SRC/grafeno.exe" "$DST/"
cp "$SRC/grafeno-cli.exe" "$DST/"
cp "assets/icon/grafeno.ico" "$DST/"
cp "assets/icon/grafeno-256.png" "$DST/"
cp "README.md" "$DST/"
cp "LICENSE-MIT" "$DST/"
cp "LICENSE-APACHE" "$DST/"

echo "Portable package: dist/windows/GrafenoVetor/"
