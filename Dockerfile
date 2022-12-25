FROM node:18-alpine as site-builder
WORKDIR /site
COPY package.json package-lock.json .
RUN npm clean-install
COPY astro.config.mjs .
COPY public/ ./public/
COPY src/pages/ ./src/pages/
RUN npm run build

FROM rust:1.66-alpine as app-builder
RUN apk --no-cache add libc-dev make openssl-dev perl pkgconfig
RUN USER=root cargo new --bin ofcrse
WORKDIR /ofcrse
COPY Cargo.lock Cargo.toml .
# Cache dependencies.
# See https://github.com/sfackler/rust-native-tls/issues/190#issuecomment-723579236.
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release
RUN rm -r src target/release/deps/ofcrse*
COPY src/*.rs ./src/
COPY --from=site-builder /site/dist/ ./dist/
RUN cargo build --release

FROM alpine:3.17
WORKDIR /app
RUN apk --no-cache add ca-certificates libgcc openssl-dev
COPY --from=app-builder /ofcrse/target/release/ofcrse .
COPY --from=site-builder /site/dist/ ./dist/
CMD ["/app/ofcrse"]
EXPOSE 3000
