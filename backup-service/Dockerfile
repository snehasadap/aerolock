# Use the official Rust image as a parent image
FROM rust:1.79

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Set the entry point for the Docker container
CMD ["cargo", "run", "--release"]

# Expose the port that your app runs on
EXPOSE 8080
