#!/bin/bash

res=results/$(date +"%y%m%d")
mkdir $res
for t in $(ls target/criterion | grep -v report); do
  mkdir $res/$t
  cp $(find target/criterion/$t -name violin.svg) $res/$t/
done
find target/criterion -name estimates.json -print -exec jq .Median.point_estimate {} \; | paste -d, - - | grep -v base >$res/median.csv
