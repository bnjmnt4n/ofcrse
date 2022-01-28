FROM node:16-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm clean-install
COPY . .
RUN npm run build

FROM pierrezemb/gostatic:latest
COPY --from=builder /app/dist /srv/http
CMD ["-port", "8080", "-enable-logging", "-https-promote"]
