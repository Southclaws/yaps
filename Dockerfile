FROM clux/muslrust:nightly as build

WORKDIR /app
COPY . .
RUN rustup default nightly && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest

COPY --from=build /app/target/x86_64-unknown-linux-musl/release/yaps /usr/local/bin/yaps
COPY --from=build /app/templates /usr/local/bin/templates

ENTRYPOINT yaps
