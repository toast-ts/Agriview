FROM rust:1.77-alpine3.19 AS compiler
ENV TZ Australia/Sydney
ENV RUSTFLAGS -C target-feature=-crt-static
RUN apk add --no-cache openssl-dev musl-dev
WORKDIR /usr/src/av
RUN cargo init
COPY . .
RUN cargo build -r

FROM alpine:3.19
RUN apk add --no-cache libgcc
WORKDIR /av
COPY --from=compiler /usr/src/av/target/release/agriview .
COPY --from=compiler /usr/src/av/templates templates/
EXPOSE 3030/tcp
CMD [ "./agriview" ]
