FROM node:18-alpine AS site-builder
ARG NPM_BUILD_COMMAND=build
WORKDIR /site
COPY site/package.json site/package-lock.json .
RUN npm clean-install
COPY site/astro.config.mjs .
COPY site/public/ ./public/
COPY site/src/ ./src/
RUN npm run $NPM_BUILD_COMMAND

FROM rust:1.70-alpine AS app-builder
RUN apk add --no-cache libc-dev make perl pkgconfig
RUN USER=root cargo new --bin ofcrse
WORKDIR /ofcrse
COPY server/Cargo.lock server/Cargo.toml .
# Cache dependencies.
RUN cargo build --release
RUN rm -r src target/release/deps/ofcrse*
COPY server/src/ ./src/
RUN cargo build --release

FROM alpine:3.18
WORKDIR /app
RUN apk add --no-cache libgcc
COPY --from=app-builder /ofcrse/target/release/ofcrse .
COPY --from=site-builder /site/dist/ ./dist/
CMD ["/app/ofcrse"]
EXPOSE 3000
