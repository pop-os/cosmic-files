#!/usr/bin/env bash

set -ex

rm -f 0*
for umode in 4 5 6 7
do
    for gmode in 0 4 5 6 7
    do
        for amode in 0 4 5 6 7
        do
            mode="0${umode}${gmode}${amode}"
            touch "${mode}"
            chmod "${mode}" "${mode}"
        done
    done
done

