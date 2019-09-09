#!/bin/bash

# Quick script to analyse binary dependencies
#
# Clearly indicates libraries outside of steam runtime, that might cause
# problems on other distributions.

list_dependencies () {
	objdump -p "$1" | \
		grep NEEDED | \
		awk '{print $2}'
}

is_in_glibc () {
	local -r found=$(rpm -ql glibc | grep "$1")
	test -n "$found"
}

find_in_dir () {
	local -r found=$(find "$1" -name "$2")
	test -n "$found"
}

find_in_steam_runtime_scout_64 () {
	find_in_dir "$HOME/.local/share/Steam/ubuntu12_32/steam-runtime/amd64/lib/x86_64-linux-gnu/" "$1"
}

find_in_fedora () {
	find_in_dir "/lib64/" "$1"
}

find_deps () {
	while read -r lib ; do
		if is_in_glibc "$lib" ; then
			printf "%25s => fedora (glibc)\n" "$lib"
		elif find_in_steam_runtime_scout_64 "$lib" ; then
			printf "%25s => steam (scout 64)\n" "$lib"
		elif find_in_fedora "$lib" ; then
			printf "\e[31m%25s => fedora\e[0m\n" "$lib"
		fi
	done
}

list_dependencies "$@" | find_deps
