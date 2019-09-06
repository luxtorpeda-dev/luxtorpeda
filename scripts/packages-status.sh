#!/bin/bash

# Print project status for all packaged projects.
#
# Count number of all tags, new commits and new tags since package release.

lowercase () {
	echo "$@" | tr '[:upper:]' '[:lower:]'
}

repo_name () {
	url=$1
	lowercase "$(basename "${url%.git}")"
}

parse_sub_status () {
	[[ "$*" =~ .([0-9a-f]+) ]] && echo "${BASH_REMATCH[1]}"
}

cd "$(git rev-parse --show-toplevel)" || exit

while read -r line ; do
	pkg=$(echo "$line" | cut -d '|' -f1 | tr -d '[:space:]')
	proj=$(echo "$line" | cut -d '|' -f2 | tr -d '[:space:]')
	mirror=$(repo_name "$proj")

	if [ -z "$pkg" ] ; then
		continue
	fi

	pkg_status=$(git -C "../packages/$pkg/" submodule status "source")
	pkg_release_version=$(parse_sub_status "$pkg_status")
	mirror_version=$(git -C "../mirrors/$mirror/" rev-parse origin/master)

	echo "package:     $pkg"

	# echo "$pkg_release_version"

	echo -n "all tags:    "
	git -C "../mirrors/$mirror/" tag | wc -l

	echo -n "new commits: "
	git -C "../mirrors/$mirror/" log \
		--oneline \
		"$pkg_release_version..$mirror_version" | wc -l

	echo "new tags:"
	git -C "../mirrors/$mirror/" log \
		--decorate \
		--pretty="format:%h %d" \
		"$pkg_release_version..$mirror_version" \
		| grep '(tag:'

	echo
done < scripts/packages.txt
