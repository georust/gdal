#!/bin/sh
# -*- coding: utf-8 -*-

set -eu

SCRIPT_DIR=$(dirname "$0")
case $SCRIPT_DIR in
    "/"*)
        ;;
    ".")
        SCRIPT_DIR=$(pwd)
        ;;
    *)
        SCRIPT_DIR=$(pwd)"/"$(dirname "$0")
        ;;
esac
GDAL_ROOT=$SCRIPT_DIR/..
cd "$GDAL_ROOT"

if ! test -d fix_typos; then
    # Get our fork of codespell that adds --words-white-list and full filename support for -S option
    mkdir fix_typos
    (cd fix_typos
     git clone https://github.com/rouault/codespell
     (cd codespell && git checkout gdal_improvements)
     # Aggregate base dictionary + QGIS one + Debian Lintian one
     curl https://raw.githubusercontent.com/qgis/QGIS/master/scripts/spell_check/spelling.dat | sed "s/:/->/" | sed "s/:%//" | grep -v "colour->" | grep -v "colours->" > qgis.txt
     curl https://salsa.debian.org/lintian/lintian/-/raw/master/data/spelling/corrections | grep "||" | grep -v "#" | sed "s/||/->/" > debian.txt
     cat codespell/data/dictionary.txt qgis.txt debian.txt | awk 'NF' > dict.txt
     echo "difered->deferred" >> dict.txt
     echo "differed->deferred" >> dict.txt
     grep -v 404 < dict.txt > dict.txt.tmp
     mv dict.txt.tmp dict.txt
    )
fi

EXCLUDED_FILES="**/.git/*,**/fix_typos/*,**/target/*,**gdal-sys/*"
AUTHORIZED_LIST=""

python3 fix_typos/codespell/codespell.py -w -i 3 -q 2 -S "$EXCLUDED_FILES,./build*/*" \
    -x script/typos_allowlist.txt --words-white-list=$AUTHORIZED_LIST \
    -D ./fix_typos/dict.txt .
