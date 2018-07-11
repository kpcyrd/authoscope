#!/bin/sh
apt-get -qq update

if [ -n "$TRAVIS" ]; then
    # update docker
    apt-get -y -o Dpkg::Options::="--force-confnew" install docker-ce
fi
