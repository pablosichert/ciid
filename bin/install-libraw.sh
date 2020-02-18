#!/bin/bash
set -euo pipefail

temp=$(mktemp -d)

git clone https://github.com/LibRaw/LibRaw $temp
pushd $temp
./mkdist.sh
./configure
make install
popd
rm -rf $temp
