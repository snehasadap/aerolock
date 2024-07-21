FROM rust:1.79
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release
CMD ["cargo", "run", "--release"]
EXPOSE 8080
