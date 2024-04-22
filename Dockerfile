FROM rust:1.77-alpine3.19 AS compiler
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache openssl-dev musl-dev
WORKDIR /usr/src/fv
RUN cargo init
COPY . .
RUN cargo fetch && cargo build -r

FROM alpine:3.19
RUN apk add --no-cache openssl-dev libgcc
WORKDIR /fv
COPY --from=compiler /usr/src/fv/target/release/field-viewer .
COPY --from=compiler /usr/src/fv/templates templates/
EXPOSE 3030/tcp
CMD [ "./field-viewer" ]
