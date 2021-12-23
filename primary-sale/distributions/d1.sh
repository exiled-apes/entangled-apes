#!/bin/bash
set -e

echo distributing 30 SOL from
solana address  -k ~/keys/exiled-custody/exiled-custody.json

echo total available before distribution
solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta

solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta Fzba5Rx6zZHzeUs7XQxaBAxMYE55qaJ8ptrobSSuPzKk 15.00000 --allow-unfunded-recipient
solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta 4PC3jH8txRfFfa2n9AtuQMY3CZ2e3Bq8sbHEG784Zp6C 15.00000 --allow-unfunded-recipient

echo total available after distribution
solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta

