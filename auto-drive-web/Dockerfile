FROM node:26-alpine AS build

WORKDIR /workspace
COPY auto-drive-web .

# Copy dependency manifests first for Docker layer caching
COPY package*.json ./

RUN npm ci

RUN npm run build

FROM nginx:alpine AS production

# Copy compiled static assets (e.g. dist/ or build/) to Nginx HTML root
COPY --from=build /workspace/dist /usr/share/nginx/html
COPY --from=build /workspace/nginx_common.conf /etc/nginx/nginx_common.conf
COPY --from=build /workspace/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
