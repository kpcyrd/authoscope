FROM alpine:edge
RUN apk add --no-cache libressl-dev
RUN apk add --no-cache --virtual .build-rust rust cargo
WORKDIR /usr/src/badtouch
COPY . .
RUN cargo build --release --verbose
RUN strip target/release/badtouch

FROM alpine:edge
RUN apk add --no-cache libgcc
COPY --from=0 /usr/src/badtouch/target/release/badtouch /usr/local/bin/badtouch
ENTRYPOINT ["badtouch"]
