FROM node:18-alpine as site-builder
WORKDIR /site
COPY package.json package-lock.json .
RUN npm clean-install
COPY astro.config.mjs .
COPY public/ ./public/
COPY site/ ./site/
RUN npm run build

FROM rust:1.66-alpine as app-builder
RUN apk add libc-dev openssl-dev pkgconfig
RUN USER=root cargo new --bin ofcrse
WORKDIR /ofcrse
COPY Cargo.lock Cargo.toml .
# Cache dependencies.
RUN cargo build --release
RUN rm -r src target/release/deps/ofcrse*
COPY src/ ./src/
RUN cargo build --release

FROM alpine:3.17
WORKDIR /app
RUN apk add openssl-dev
COPY --from=app-builder /ofcrse/target/release/ofcrse .
COPY --from=site-builder /site/dist/ ./dist/
CMD ["/app/ofcrse"]
