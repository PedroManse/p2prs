#! /usr/bin/env bash
if [ "$1" = "--allow-dirty" ] || [ "$2" = "--allow-dirty" ] ; then allow_dirty="--allow-dirty" ; fi
if [ "$1" = "--fix" ] || [ "$2" = "--fix" ] ; then fix="--fix" ; fi

set -ex
cargo build
cargo fmt
cargo clippy $fix $allow_dirty --all-features -- \
	-Dclippy::perf \
	-Dclippy::style \
	-Wclippy::pedantic \
	-Aclippy::enum_glob_use \
	-Aclippy::too_many_lines \
	-Aclippy::match_same_arms \
	-Aclippy::wildcard_imports \
	-Aclippy::unnecessary_wraps \
	-Aclippy::unnested_or_patterns \
	-Aclippy::cast_possible_truncation \
	-Aclippy::missing_errors_doc
cargo test
