FROM rust:latest
WORKDIR /usr/src/badtouch
COPY . .
RUN cargo build --release --verbose \
    && strip target/release/badtouch
FROM busybox:1-glibc
COPY --from=0 /usr/src/badtouch/target/release/badtouch /usr/local/bin/badtouch
COPY --from=0 /usr/lib/x86_64-linux-gnu/libssl.so.1.1 \
    /usr/lib/x86_64-linux-gnu/libcrypto.so.1.1 \
    /lib/x86_64-linux-gnu/libdl.so.2 \
    /lib/x86_64-linux-gnu/librt.so.1 \
    /lib/x86_64-linux-gnu/libgcc_s.so.1 \
    /lib/x86_64-linux-gnu/
ENTRYPOINT ["badtouch"]
