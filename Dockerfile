FROM clux/muslrust:stable-2024-05-01 AS chef
RUN cargo install cargo-chef
WORKDIR /usr/src/av

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /usr/src/av/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build -r --target x86_64-unknown-linux-musl

FROM alpine:3.19
COPY --from=builder /usr/src/av/target/x86_64-unknown-linux-musl/release/agriview .
CMD ./agriview
