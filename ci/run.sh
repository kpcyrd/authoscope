#!/bin/sh
set -exu
case "$BUILD_MODE" in
    build)
        cargo build --verbose --all
        cargo build --verbose --examples
        cargo test --verbose --all
        ;;
    integration)
        echo "[*] building badtouch"
        cargo build --verbose --all

        echo "[*] testing smtp"
        docker build -t badtouch-smtpd ci/smtp/
        docker run --name badtouch-smtpd -d --rm -p 127.0.0.1:25:25 badtouch-smtpd

        echo root@example.com:foo > /tmp/badtouch-smtp-input.txt
        target/debug/badtouch -o badtouch-smtp-output.txt creds /tmp/badtouch-smtp-input.txt scripts/smtp.lua
        grep root@example.com badtouch-smtp-output.txt

        docker stop badtouch-smtpd
        ;;
esac
