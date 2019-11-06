#!/bin/sh
case "$TRAVIS_OS_NAME" in
    linux)
        sudo apt-get -qq update
        if [ -n "$TRAVIS" ]; then
            # update docker
            sudo apt-get -y -o Dpkg::Options::="--force-confnew" install docker-ce
        fi
        ;;
esac
