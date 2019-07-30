#!/bin/env bash

lowercase () {
	echo "$@" | tr '[:upper:]' '[:lower:]'
}

repo_name () {
	url=$1
	lowercase "$(basename "${url%.git}")"
}

readonly all_packages="ioq3 iortcw openjk openxcom"

# set -x

cd "$(git rev-parse --show-toplevel)" || exit
if [ ! -d ../packages ] ; then
	mkdir -p ../packages
fi
cd ../packages || exit
echo "Using dir: $(pwd)"

# initializing:

if [ ! -d package-template ] ; then
	git clone git@gitlab.com:luxtorpeda/package-template.git
fi

for name in $all_packages ; do
	if [ ! -d "$name" ] ; then
		git clone git@gitlab.com:luxtorpeda/packages/${name}.git
		# TODO submodule init, possibly with --reference
	fi
done
