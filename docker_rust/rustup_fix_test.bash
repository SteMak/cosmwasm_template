#!/bin/bash

cd $(dirname $0)
(./rustup_output_sample.bash 2> >(./info_to_stdout.bash)) 1>ok 2>err
sleep 0.5

echo ok
cat ok
rm ok
echo
echo err
cat err
rm err
