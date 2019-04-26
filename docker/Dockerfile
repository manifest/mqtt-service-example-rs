## -----------------------------------------------------------------------------
## Build
## -----------------------------------------------------------------------------
FROM clux/muslrust:nightly-2019-04-16 as build-stage

WORKDIR "/build"
COPY . .
RUN cargo build --release

## -----------------------------------------------------------------------------
## Package
## -----------------------------------------------------------------------------
FROM alpine

COPY --from=build-stage "/build/target/x86_64-unknown-linux-musl/release/mqtt-svc-ex" "/app/mqtt-svc-ex"

WORKDIR "/app"
ENTRYPOINT ["/app/mqtt-svc-ex"]
