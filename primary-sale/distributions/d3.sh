#!/bin/bash
set -e

echo distributing 43.0363 SOL from
solana address  -k ~/keys/exiled-custody/exiled-custody.json

echo total available before distribution
solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta

solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u https://ssc-dao.genesysgo.net Fzba5Rx6zZHzeUs7XQxaBAxMYE55qaJ8ptrobSSuPzKk 21.51815 --allow-unfunded-recipient
solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u https://ssc-dao.genesysgo.net 4PC3jH8txRfFfa2n9AtuQMY3CZ2e3Bq8sbHEG784Zp6C 21.51815 --allow-unfunded-recipient

echo total available after distribution
solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta

