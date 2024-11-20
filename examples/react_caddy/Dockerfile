# Stage 1: Build the React app
FROM node:22-alpine AS builder

WORKDIR /app

# Install dependencies
COPY package.json package-lock.json ./
RUN npm install

# Build the Vite app
COPY . .
RUN npm run build

# Stage 2: Serve with Caddy
FROM caddy:2.7-alpine

WORKDIR /srv

# Copy the built app from the builder stage
COPY --from=builder /app/dist /srv

# Copy a basic Caddyfile for serving the app
COPY Caddyfile /etc/caddy/Caddyfile
