
# xape-miner

Basically this is a quick tool to load the mints and meta data
for the mirc xapes and mono xapes so we can build the commands
to correctly entangle them.

## Usage


```bash
cargo run --quiet -- load-mints \
    --db ../data/mine.db \
    --mirc-file ../data/mirc-exile-mints.log \
    --mono-file ../data/mono-exile-mints.log \
    --rpc https://ssc-dao.genesysgo.net

cargo run --quiet -- load-blanks \
    --db ../data/mine.db \
    --csv-file ../data/blanks.csv

cargo run --quiet -- load-entanglements \
    --db ../data/mine.db

 sqlite3 ../data/mine.db 'select mirc_mints.inmate_number, mirc_mints.meta_name as mirc_meta_name, mono_mints.meta_name as mono_meta_name, mirc_mint_address, mono_mint_address, mirc_mints.meta_uri as mirc_meta_uri, mono_mints.meta_uri as mono_meta_uri, mirc_mints.image_uri as mirc_image_uri, mono_mints.image_uri as mono_image_uri from entanglements join mirc_mints on mirc_mints.mint_address = entanglements.mirc_mint_address join mono_mints on mono_mints.mint_address = entanglements.mono_mint_address order by cast(mirc_mints.inmate_number as number)' --header --csv  > entanglements.csv
 ```