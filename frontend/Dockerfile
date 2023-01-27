FROM node:12
WORKDIR /app
COPY ./src ./
COPY ./public ./
COPY ./package*.json ./
RUN npm install
RUN npm run build

FROM nginx:latest
COPY ./nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/build /frontend
EXPOSE 80