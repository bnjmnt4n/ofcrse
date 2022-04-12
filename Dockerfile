FROM node:16-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm clean-install
COPY . .
RUN npm run build

FROM openresty/openresty:1.19.9.1-10-alpine
ADD nginx.conf /usr/local/openresty/nginx/conf/nginx.conf
COPY --from=builder /app/dist /var/www/ofcr.se
ENV NGINX_PORT=8080
