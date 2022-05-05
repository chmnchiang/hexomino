FROM rust:latest as builder

WORKDIR /usr/src/app
RUN cargo install --locked trunk
RUN rustup toolchain install nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
COPY . .
RUN trunk build --release
RUN cargo build --bin hexomino-server --release

FROM debian:bullseye-slim
RUN useradd -ms /bin/bash app
USER app
WORKDIR /app
ENV SERVER_ADDR="0.0.0.0:3000"
COPY --from=builder /usr/src/app/target/release/hexomino-server /app/hexomino-server
COPY --from=builder /usr/src/app/dist /app/dist
CMD ["./hexomino-server"]
EXPOSE 3000
