# Dockerfile
FROM rust:latest AS builder

WORKDIR /app

# Copia todo el proyecto (mejor usar .dockerignore para excluir target, etc.)
COPY . .

# Compila en modo release
RUN cargo build --release

# (Opcional, si quieres una imagen más pequeña)
# FROM debian:bullseye-slim
# COPY --from=builder /app/target/release/app /usr/local/bin/app
# WORKDIR /usr/local/bin/

# Añade este comando para mantener el contenedor vivo
CMD ["tail", "-f", "/dev/null"]
