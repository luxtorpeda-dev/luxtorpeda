#!/bin/bash

# Setup working copies for projects mirrored by Luxtorpeda project.
# Perform one way sync (origin → gitlab) of master branches.

lowercase () {
	echo "$@" | tr '[:upper:]' '[:lower:]'
}

repo_name () {
	url=$1
	lowercase "$(basename "${url%.git}")"
}

list_projects () {
	awk -F '|' '{print $2}' scripts/packages.txt
}

cd "$(git rev-parse --show-toplevel)" || exit

readonly all_projects=$(list_projects)

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
	git -C "$repo_name" checkout master
 	git -C "$repo_name" push gitlab master
	echo
done

# syncing

for project_url in $all_projects ; do
	repo_name="$(repo_name "$project_url")"
	echo "Syncing $repo_name"
	git -C "$repo_name" fetch --all
	git -C "$repo_name" push --force gitlab origin/master:master
 	git -C "$repo_name" push --tags gitlab
	echo
done
