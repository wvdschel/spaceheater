#!/bin/bash
set -e

if [[ -z "$1" ]]; then
  echo usage ./benchmark_host.sh user@hostname
  exit 1
fi

ssh ${1} bash <install.sh
ssh ${1} /home/topsnek/topsnek/target/release/replay spaceheater3 < sample_games/4_players_11x11_wrapped_royale.json.gz | tee royale_4_player_${1}.log