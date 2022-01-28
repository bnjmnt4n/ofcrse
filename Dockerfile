FROM node:16-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm clean-install
COPY . .
RUN npm run build

FROM nginx:alpine
ADD nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/dist /var/www/ofcr.se
ENV NGINX_PORT=8080
