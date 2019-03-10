FROM rustlang/rust:nightly as build

WORKDIR /app
COPY . .
RUN rustup default nightly && \
    cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest

COPY --from=build /app/target/x86_64-unknown-linux-musl/release/yaps /usr/local/bin/yaps

ENTRYPOINT yaps
