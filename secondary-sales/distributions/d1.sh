#!/bin/bash
set -e

echo distributing 16.4937 SOL from
solana address  -k ~/keys/exiled-custody/exiled-custody.json

echo total available before distribution
solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta

# solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u https://ssc-dao.genesysgo.net Fzba5Rx6zZHzeUs7XQxaBAxMYE55qaJ8ptrobSSuPzKk 4.56710 --allow-unfunded-recipient
# solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u https://ssc-dao.genesysgo.net 4PC3jH8txRfFfa2n9AtuQMY3CZ2e3Bq8sbHEG784Zp6C 8.37385 --allow-unfunded-recipient
solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u https://ssc-dao.genesysgo.net H6GSnttdzaY9xuNcCD6uQf3tdwWKJoiWHEy5xoQCdi4A 3.55274 --allow-unfunded-recipient

echo total available after distribution
solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta
