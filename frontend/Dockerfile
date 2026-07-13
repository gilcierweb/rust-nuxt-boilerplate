# -- Development 
FROM node:22-alpine AS development

WORKDIR /app

RUN corepack enable

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .

EXPOSE 3000

CMD ["pnpm", "dev", "--host", "0.0.0.0"]

# -- Build 
FROM node:22-alpine AS builder

WORKDIR /app

RUN corepack enable

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .

RUN pnpm build

# -- Production 
FROM node:22-alpine AS production

RUN addgroup -S nuxt && adduser -S nuxt -G nuxt

WORKDIR /app

COPY --from=builder /app/.output ./.output

RUN chown -R nuxt:nuxt /app
USER nuxt

EXPOSE 3000

ENV NITRO_HOST=0.0.0.0
ENV NITRO_PORT=3000
ENV NODE_ENV=production

HEALTHCHECK --interval=30s --timeout=5s \
    CMD wget -qO- http://localhost:3000/ || exit 1

CMD ["node", ".output/server/index.mjs"]
