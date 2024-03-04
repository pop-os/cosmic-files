#!/usr/bin/env bash

set -e

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
    if [ -z "${default}" ]
    then
        echo "${file}: ${filetype}: no default application found"
        exit 1
    fi

    echo "${file}: ${filetype}: ${default}"
done
