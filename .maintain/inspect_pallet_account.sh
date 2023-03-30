#!/usr/bin/env bash
set -e

PALLET_ID=${1?"PalletId must be passed"}
NETWORK=${2:-"bajun"}

[ ${#PALLET_ID} -lt 8 ] && (
    echo "PalletId must be of 8 or more characters" 1>&2
    exit 1
)

hex=$(printf "modl$PALLET_ID" | xxd -p)
public_key=$(printf 0x%-64s $hex | tr " " 0)
subkey inspect $public_key --network=$NETWORK --public
