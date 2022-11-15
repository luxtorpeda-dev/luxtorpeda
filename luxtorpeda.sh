#!/bin/bash

if [[ ! -z "${LD_PRELOAD}" ]]; then
    echo "LD_PRELOAD found with $LD_PRELOAD"
    export ORIGINAL_LD_PRELOAD="$LD_PRELOAD"
    export LD_PRELOAD=""
fi

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

if [[ $2 == *"iscriptevaluator.exe"* ]]; then
  echo "ignoring iscriptevaluator.exe"
  exit 0
fi

"$DIR/luxtorpeda.x86_64" "$@"
