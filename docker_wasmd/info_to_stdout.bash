#!/bin/bash

while read line; do
  if [[ $(echo $line | cut -c1-15) == "go: downloading" ]] ||
    [[ $(echo $line | cut -c1-12) == "Cloning into" ]] ||
    [[ $(echo $line | cut -c1-14) == "HEAD is now at" ]]; then
    echo "$line" 1>&1
  else
    echo "$line" 1>&2
  fi
done
