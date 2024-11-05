# Шаг 1: Создаем builder-слой для сборки приложения
FROM rust:1.81 as builder

# Устанавливаем рабочую директорию
WORKDIR /usr/src/app

# Копируем только файлы Cargo.toml и Cargo.lock для кэширования зависимостей
COPY Cargo.toml Cargo.lock ./

# Копируем Cargo.toml для каждого пакета в воркспейсе
COPY server/Cargo.toml server/Cargo.toml
RUN mkdir server/src && echo "fn main() {}" > server/src/main.rs
COPY cli/Cargo.toml cli/Cargo.toml
RUN mkdir cli/src && echo "fn main() {}" > cli/src/main.rs
COPY shared/Cargo.toml shared/Cargo.toml
RUN mkdir shared/src && echo "fn main() {}" > shared/src/lib.rs

# Скачиваем и собираем зависимости для воркспейса
RUN cargo build --release --workspace && rm -rf target/release/deps/*app*

# Копируем исходный код приложения
COPY . .

# Собираем целевой пакет в релизном режиме (замените `your_app_name` на имя вашего пакета)
RUN cargo build --release --package server

# Шаг 2: Создаем минимальный runtime-слой
FROM debian:bookworm-slim 

# Устанавливаем основные зависимости
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Копируем собранный бинарник из builder-слоя
COPY --from=builder /usr/src/app/target/release/server /usr/local/bin/server

# Устанавливаем переменную среды
ENV RUST_LOG=info
EXPOSE 8081

# Запускаем приложение
CMD ["server"]

