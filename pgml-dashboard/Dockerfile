FROM rust:1
RUN cargo install sqlx-cli
RUN apt-get update && apt-get install -y nodejs npm
RUN npm install -g sass
COPY . /app
WORKDIR /app
