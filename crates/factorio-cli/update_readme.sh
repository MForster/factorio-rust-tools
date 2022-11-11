#!/bin/bash
IFS=''
while read -r line; do
  end='<!-- END EMBED -->'
  if [[ $line =~ $end ]]; then
    skipping=false
  fi

  if [[ $skipping != true ]]; then
    echo "$line"
  fi

  start='<!-- EMBED: (.*) -->'
  if [[ $line =~ $start ]]; then
    cmd=${BASH_REMATCH[1]}
    echo '```sh'
    echo "\$ $cmd"
    eval ${cmd/fct/cargo -q run -- }
    echo '```'
    skipping=true
  fi

done \
  <README.md | sponge README.md
