#!/bin/bash

while read line; do
  if [[ $(echo $line | cut -c1-5) == "info:" ]]; then
    echo "$line" 1>&1
  else
    echo "$line" 1>&2
  fi
done
