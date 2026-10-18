#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

usage() {
  cat <<'USAGE'
Usage: ./do <command>

  build   Build the four programs, their IDLs and TS types
  test    Build, then run the unit tests and the LiteSVM integration tests
  check   rustfmt --check and clippy with warnings as errors
  fmt     Format all Rust code
  idl     Build and copy the IDLs and TS types into client/src/idl
  keys    Sync program IDs with target/deploy keypairs (after a fresh clone, to deploy your own copy)
  help    Show this help
USAGE
}

cmd="${1:-help}"
case "$cmd" in
  build)
    anchor build
    ;;
  test)
    anchor build
    cargo test --workspace
    ;;
  check)
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    ;;
  fmt)
    cargo fmt --all
    ;;
  idl)
    anchor build
    cp target/idl/*.json client/src/idl/
    cp target/types/*.ts client/src/idl/
    ;;
  keys)
    anchor keys sync
    governance_id="$(anchor keys list | awk '$1 == "governance:" { print $2 }')"
    sed -i "s/GOVERNANCE_PROGRAM_ID: Pubkey = pubkey!(\"[^\"]*\")/GOVERNANCE_PROGRAM_ID: Pubkey = pubkey!(\"$governance_id\")/" \
      programs/asset/src/constants.rs
    echo "asset::GOVERNANCE_PROGRAM_ID = $governance_id"
    ;;
  help | -h | --help)
    usage
    ;;
  *)
    echo "unknown command: $cmd" >&2
    usage >&2
    exit 1
    ;;
esac
