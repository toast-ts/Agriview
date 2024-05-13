FROM clux/muslrust:1.78.0-stable AS chef
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
COPY --from=builder /usr/src/av/templates templates/
EXPOSE 3030/tcp
CMD ./agriview
