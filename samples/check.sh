#!/usr/bin/env bash

set -e

cd "$(dirname "$0")"

for file in */*
do
    filetype="$(xdg-mime query filetype "${file}")"
    if [ -z "${filetype}" ]
    then
        echo "${file}: no filetype found"
        exit 1
    fi
    if [ "${file%.*}" != "${filetype}" ]
    then
        echo "${file} is not named according to filetype ${filetype}"
        exit 1
    fi

    default="$(xdg-mime query default "${filetype}")"
    if [ -n "${default}" ]
    then
        echo "${file}: ${filetype}: ${default}"
    else
        echo "${file}: ${filetype}: no default application found"
    fi
done
