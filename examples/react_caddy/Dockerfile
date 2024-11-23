# Stage 1: Build the React app
FROM node:22-alpine AS builder

# Определяем аргумент сборки
ARG VITE_DOMAIN

WORKDIR /app

# Install dependencies
COPY package.json package-lock.json ./
RUN npm install

# Build the Vite app
COPY . .

ENV VITE_DOMAIN=${VITE_DOMAIN}
RUN echo "VITE_DOMAIN=${VITE_DOMAIN}" >> .env
RUN npm run build

# Stage 2: Serve with Caddy
FROM caddy:2.7-alpine

WORKDIR /srv

ARG VITE_DOMAIN
ENV VITE_DOMAIN=${VITE_DOMAIN}

# Copy the built app from the builder stage
COPY --from=builder /app/dist /srv
COPY --from=builder /app/.env /srv

# Copy a basic Caddyfile for serving the app
COPY Caddyfile /etc/caddy/Caddyfile

CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
