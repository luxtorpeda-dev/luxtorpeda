#!/bin/env bash

# Setup working copies for projects mirrored by Luxtorpeda project.
# Perform one way sync (origin → gitlab) of master branches.

lowercase () {
	echo "$@" | tr '[:upper:]' '[:lower:]'
}

repo_name () {
	url=$1
	lowercase "$(basename "${url%.git}")"
}

readonly ioq3_url=git@github.com:ioquake/ioq3.git
readonly openjk_url=git@github.com:JACoders/OpenJK.git
readonly openxcom_url=git@github.com:OpenXcom/OpenXcom.git

readonly all_projects="$ioq3_url $openjk_url $openxcom_url"

# set -x

cd "$(git rev-parse --show-toplevel)" || exit
if [ ! -d ../mirrors ] ; then
	mkdir -p ../mirrors
fi
cd ../mirrors || exit
echo "Using dir: $(pwd)"

# initializing:

for project_url in $all_projects ; do
	repo_name="$(repo_name "$project_url")"
	mirror_url=git@gitlab.com:luxtorpeda/mirrors/${repo_name}.git
	if [ -d "$repo_name" ] ; then
		continue
	fi
	echo "Cloning $project_url"
	git clone "$project_url" "$repo_name"
	git -C "$repo_name" remote add gitlab "$mirror_url"
 	git -C "$repo_name" push gitlab master
done

# syncing

for project_url in $all_projects ; do
	repo_name="$(repo_name "$project_url")"
	echo "Syncing $repo_name"
	git -C "$repo_name" fetch --all
	git -C "$repo_name" push --force gitlab origin/master:master
done
