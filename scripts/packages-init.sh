#!/bin/bash

# Clone and initialize all Luxtorpeda packages.

set -e

list_packages () {
	awk -F '|' '{print $1}' scripts/packages.txt
}

cd "$(git rev-parse --show-toplevel)"

readonly all_packages=$(list_packages)

if [ ! -d ../packages ] ; then
	mkdir -p ../packages
fi
cd ../packages || exit
echo "Using dir: $(pwd)"

# initializing:

if [ ! -d template ] ; then
	git clone git@gitlab.com:luxtorpeda/packages/template.git
fi

for name in $all_packages ; do
	if [ ! -d "$name" ] ; then
		git clone "git@gitlab.com:luxtorpeda/packages/${name}.git"
		git -C "$name" remote add template git@gitlab.com:luxtorpeda/packages/template.git
		git -C "$name" submodule init
		git -C "$name" fetch --all
	fi
done
