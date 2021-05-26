FROM rust:alpine3.13
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add musl-dev openssl-dev
WORKDIR /app
COPY . .
RUN cargo build --release --locked --verbose
RUN strip target/release/authoscope

FROM alpine:3.13
RUN apk add --no-cache libgcc openssl
COPY --from=0 /app/target/release/authoscope /usr/local/bin/authoscope
ENTRYPOINT ["authoscope"]
