#!/bin/env bash

lowercase () {
	echo "$@" | tr '[:upper:]' '[:lower:]'
}

repo_name () {
	url=$1
	lowercase "$(basename "${url%.git}")"
}

readonly all_packages="arxlibertatis dhewm3 eternaljk gzdoom ioq3 iortcw openjk openmw openrct2 openxcom"

# set -x

cd "$(git rev-parse --show-toplevel)" || exit
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
