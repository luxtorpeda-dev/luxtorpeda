#!/bin/bash

if [[ ! -z "${LD_PRELOAD}" ]]; then
    echo "LD_PRELOAD found with $LD_PRELOAD"
    export ORIGINAL_LD_PRELOAD="$LD_PRELOAD"
    export LD_PRELOAD=""
fi

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

"$DIR/luxtorpeda" "$@"
