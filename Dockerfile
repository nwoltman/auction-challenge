FROM rust:1.37

# Trickery to make builds faster if source doesn't change
RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY Cargo.toml Cargo.toml
RUN cargo build --release
RUN rm ./target/*/deps/auction_challenge*

COPY src src
RUN cargo build --release

CMD ["cargo", "run", "--release", "--"]
