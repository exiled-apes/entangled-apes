#!/bin/bash

for i in {0..517}
do
  name="ExiledApe $(($i+1))/518"
  jq --arg name "$name" '.name=$name' assets/$i.json | sponge assets/$i.json
done
