#!/bin/bash
set -euo pipefail

temp=$(mktemp -d)

git clone https://github.com/exiftool/exiftool $temp
pushd $temp
perl Makefile.PL
make
make test
make install
popd
rm -rf $temp
