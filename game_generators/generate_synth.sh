#!/bin/bash

mkdir "./games/synth"
mkdir "./games/synth/p0.0"
mkdir "./games/synth/p0.1"

cargo build --release --bin synthetic

for p in "0.1" "0.0"; do
for i in {1..10}; do

	echo "$i";
	echo "$p";

	# Generate small games
	RUST_LOG=debug ./target/release/synthetic -M 2 -n 2 2 -m 2 2 -p $p -u 0 1 -v 0 1 -r "$i" -o "./games/synth/p$p/small_no_subgames$i.game" -b "./games/synth/p$p/small_no_subgames$i.vec"
	RUST_LOG=debug ./target/release/synthetic -M 2 -n 2 2 -m 2 2 -p $p -u 0 1 -v 0 1 -r "$i" -s -o "./games/synth/p$p/small_with_subgames$i.game" -b "./games/synth/p$p/small_with_subgames$i.vec"

	# Generate medium games
	RUST_LOG=debug ./target/release/synthetic -M 10 -n 2 2 -m 10 10 -p "$p" -u 0 1 -v 0 1 -r "$i" -o "./games/synth/p$p/medium_no_subgames$i.game" -b "./games/synth/p$p/medium_no_subgames$i.vec"
	RUST_LOG=debug ./target/release/synthetic -M 10 -n 2 2 -m 10 10 -p "$p" -u 0 1 -v 0 1 -r "$i" -s -o "./games/synth/p$p/medium_with_subgames$i.game" -b "./games/synth/p$p/medium_with_subgames$i.vec"

	# Generate large games
	RUST_LOG=debug ./target/release/synthetic -M 100 -n 2 2 -m 100 100 -p "$p" -u 0 1 -v 0 1 -r "$i" -o "./games/synth/p$p/large_no_subgames$i.game" -b "./games/synth/p$p/large_no_subgames$i.vec"
	RUST_LOG=debug ./target/release/synthetic -M 100 -n 2 2 -m 100 100 -p "$p" -u 0 1 -v 0 1 -r "$i" -s -o "./games/synth/p$p/large_with_subgames$i.game" -b "./games/synth/p$p/large_with_subgames$i.vec"

	# Generate huge games
	RUST_LOG=debug ./target/release/synthetic -M 100 -n 5 5 -m 100 100 -p "$p" -u 0 1 -v 0 1 -r "$i" -o "./games/synth/p$p/huge_no_subgames$i.game" -b "./games/synth/p$p/huge_no_subgames$i.vec"
	RUST_LOG=debug ./target/release/synthetic -M 100 -n 5 5 -m 100 100 -p "$p" -u 0 1 -v 0 1 -r "$i" -s -o "./games/synth/p$p/huge_with_subgames$i.game" -b "./games/synth/p$p/huge_with_subgames$i.vec"

done
done
