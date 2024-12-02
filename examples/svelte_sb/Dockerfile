FROM node:22-slim

WORKDIR /app

COPY package.json package-lock.json ./

RUN npm install --frozen-lockfile

COPY . .

ARG VITE_POCKETBASE_URL
ENV VITE_POCKETBASE_URL=${VITE_POCKETBASE_URL}

RUN npm run build

ENV NODE_ENV production

EXPOSE 3000

CMD ["node", "build"]
