FROM rustlang/rust:nightly as build

WORKDIR /app
COPY . .
RUN rustup default nightly && cargo build --release

FROM alpine:latest

COPY --from=build /app/target/release/yaps /usr/local/bin/yaps

ENTRYPOINT ["yaps"]
