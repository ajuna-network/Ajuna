#!/bin/bash

# using default port if none given as arguments
NODE_PORT=9944
WORKER_PORT=2011
TARGET_IP="127.0.0.1"

echo "Using node-port ${NODE_PORT}"
echo "Using trusted-worker-port ${WORKER_PORT}"

BALANCE=1000
TRANSFER_BALANCE=100

CLIENT="./integritee-cli -p ${NODE_PORT} -P ${WORKER_PORT} -u ws://${TARGET_IP} -U wss://${TARGET_IP}"

echo "Reading MRENCLAVE..."
read -r MRENCLAVE <<< "$($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')"
if [[ -z "$MRENCLAVE" ]]; then
  echo -e "Failed to read MRENCLAVE exiting...\n"
  exit 1
else
  echo -e "Read MRENCLAVE from worker list: ${MRENCLAVE}\n"
fi

echo "* Creating account for Alice"
ACCOUNT_ALICE=//Alice
echo -e "--> Alice's account = ${ACCOUNT_ALICE}\n"

echo "* Creating account for Bob"
ACCOUNT_BOB=//Bob
echo -e "-->  Bob's account = ${ACCOUNT_BOB}\n"

echo "* Issuing ${BALANCE} tokens to Alice's account"
if ! ${CLIENT} trusted --mrenclave "${MRENCLAVE}" --direct set-balance "${ACCOUNT_ALICE}" "${BALANCE}"; then
   echo -e "Command failed exiting...\n"
   exit 1
else
   echo -e "Issuing successful! Sleeping for 1 second\n"
fi
sleep 1

echo "* Issue ${BALANCE} tokens to Bob's account"
if ! ${CLIENT} trusted --mrenclave "${MRENCLAVE}" --direct set-balance "${ACCOUNT_BOB}" "${BALANCE}"; then
   echo -e "Command failed exiting...\n"
   exit 1
else
   echo -e "Issuing successful! Sleeping for 1 second\n"
fi
sleep 1

echo "* Transferring ${TRANSFER_BALANCE} tokens from Alice to Bob's account"
if ! ${CLIENT} trusted --mrenclave "${MRENCLAVE}" --direct transfer "${ACCOUNT_ALICE}" "${ACCOUNT_BOB}" "${TRANSFER_BALANCE}"; then
   echo -e "Command failed exiting...\n"
   exit 1
else
   echo -e "Transfer successful! Sleeping for 1 second\n"
fi
sleep 1

echo "* Transferring ${TRANSFER_BALANCE} tokens from Bob to Alices's account"
if ! ${CLIENT} trusted --mrenclave "${MRENCLAVE}" --direct transfer "${ACCOUNT_BOB}" "${ACCOUNT_ALICE}" "${TRANSFER_BALANCE}"; then
   echo -e "Command failed exiting...\n"
   exit 1
else
   echo -e "Transfer successful! Sleeping for 1 second\n"
fi
sleep 1

echo -e "All tests succeeded! Exiting...\n"
exit 0
