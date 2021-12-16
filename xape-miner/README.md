
# xape-miner

Basically this is a quick tool to load the mints and meta data
for the mirc xapes and mono xapes so we can build the commands
to correctly entangle them.

## Usage

Load up the mirc_mints and mono_mints tables:

```bash
cargo run --quiet -- \
    load-mints \
    --mirc-file ../data/mirc-exile-mints.log \
    --mono-file ../data/mono-exile-mints.log \
    --db ../data/mine.db \
    --rpc https://ssc-dao.genesysgo.net
```