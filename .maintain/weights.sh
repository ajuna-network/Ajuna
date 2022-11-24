#!/bin/bash

RUNTIME=${1?Either bajun or ajuna must be passed}

cargo build-"${RUNTIME}"-benchmarks

PALLETS=(
  "cumulus-pallet-xcmp-queue"
  "frame-system"
  "pallet-balances"
  "pallet-collator-selection"
  "pallet-collective"
  "pallet-identity"
  "pallet-membership"
  "pallet-multisig"
  "pallet-preimage"
  "pallet-proxy"
  "pallet-scheduler"
  "pallet-session"
  "pallet-timestamp"
  "pallet-treasury"
  "pallet-utility"
)

cd "$(git rev-parse --show-toplevel)" || exit
for PALLET in "${PALLETS[@]}"; do
  ./target/release/"${RUNTIME}"-para benchmark pallet \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    --pallet="${PALLET}" \
    --extrinsic="*" \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --header="./HEADER-AGPL" \
    --output="./runtime/${RUNTIME}/src/weights/${PALLET//-/_}.rs"
done

[ RUNTIME != "bajun" ] && exit 0
CUSTOM_PALLETS=(
  "pallet-ajuna-awesome-avatars"
)
for PALLET in "${CUSTOM_PALLETS[@]}"; do
  ./target/release/"${RUNTIME}"-para benchmark pallet \
    --chain=dev \
    --steps=50 \
    --repeat=20 \
    --pallet="${PALLET}" \
    --extrinsic="*" \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --template="./.maintain/frame-weight-template.hbs" \
    --output="./pallets/${PALLET#pallet-}/src/weights.rs"
done
