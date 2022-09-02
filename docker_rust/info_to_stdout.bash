#!/bin/bash

# Here we take lines and process them to stdout if they are 100% info
while read line; do
  if [[ $(echo $line | cut -c1-5) == "info:" ]]; then
    echo "$line" 1>&1
  else
    echo "$line" 1>&2
  fi
done
