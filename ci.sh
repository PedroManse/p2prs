#! /usr/bin/sh

set -ex
cargo build
cargo fmt
cargo clippy --fix --allow-dirty --all-targets --all-features -- \
	-Dclippy::perf \
	-Dclippy::style \
	-Wclippy::pedantic \
	-Aclippy::enum_glob_use \
	-Aclippy::too_many_lines \
	-Aclippy::match_same_arms \
	-Aclippy::wildcard_imports \
	-Aclippy::unnecessary_wraps \
	-Aclippy::missing_errors_doc \
	-Aclippy::cast_possible_truncation
	-Aclippy::unnested_or_patterns \
	-Aclippy::cast_possible_truncation \
	-Aclippy::missing_errors_doc
cargo test
