#!/bin/bash

# Result in stderr
# Next line should be filtered to stdout
echo "info: warn" 1>&2
echo "not info: warn" 1>&2
echo "not: warn" 1>&2

# Result in stdout
echo "info: info"
echo "not info: info"
echo "not: info"
