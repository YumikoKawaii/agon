-include .env

.PHONY: up down build logs gen client play

up:
	docker compose up --build -d

down:
	docker compose down

build:
	docker compose build --no-cache

logs:
	docker compose logs -f app

gen:
	buf generate

client:
	cargo build --release

play:
	cargo run --release
