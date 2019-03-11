FROM clux/muslrust:nightly as build

WORKDIR /app
COPY . .
RUN rustup default nightly && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest

WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/yaps /app/yaps
COPY --from=build /app/templates /app/templates
COPY --from=build /app/static /app/static

ENTRYPOINT ./yaps
