FROM rustlang/rust:nightly as build

WORKDIR /app
COPY . .
RUN rustup default nightly && cargo build --release

FROM scratch

COPY --from=build /app/target/release/yaps /bin/yaps

ENTRYPOINT ["yaps"]
