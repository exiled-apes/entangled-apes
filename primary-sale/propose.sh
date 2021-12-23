#!/bin/bash
set -e

wallet1="Fzba5Rx6zZHzeUs7XQxaBAxMYE55qaJ8ptrobSSuPzKk"; share1=`echo "$1 * 0.50000" | bc` ## mirc
wallet2="4PC3jH8txRfFfa2n9AtuQMY3CZ2e3Bq8sbHEG784Zp6C"; share2=`echo "$1 * 0.50000" | bc` ## exiled custody

echo "#!/bin/bash"
echo "set -e"
echo

echo "echo distributing $1 SOL from"
echo "solana address  -k ~/keys/exiled-custody/exiled-custody.json"
echo

echo "echo total available before distribution"
echo "solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta"
echo

echo "solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta $wallet1 $share1 --allow-unfunded-recipient"
echo "solana transfer -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta $wallet2 $share2 --allow-unfunded-recipient"
echo

echo "echo total available after distribution"
echo "solana balance  -k ~/keys/exiled-custody/exiled-custody.json -u mainnet-beta"
echo
