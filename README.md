# TKL Chat — A Cross-Platform Chat Room

**TKL Chat** is a modern, cross-platform chat room application built with a Rust backend and TypeScript frontend. It supports real-time messaging via WebSockets and is containerised using Docker for easy setup.

## Getting Started

Follow the steps below to get up and running with TKL Chat:

> ⚠️ Make sure you've defined all required fields from `.env.example` in your `.env` file before proceeding.

### 1. Start Docker Containers

```bash
docker compose -f "compose.yaml" -p "chat" up -d
```

This command will launch all necessary services defined in `compose.yaml`. *Alternatively, to run the application without uploading images to Docker Hub, there is a separate Docker Compose file. To run the dev version:*

```bash
docker compose -f "compose.dev.yaml" -p "chat-dev" up -d
```

> ⚠️ Without nginx running on Docker, the ports for each service will differ while developing, please keep this in mind.

### 2. Access the Application

Now, from your browser or the TKL Chat app, you can run the application correctly. The API is accessible at `127.0.0.1:8080`.

## Initialising the Database (not required if running Docker Compose)

```bash
cargo install diesel_cli --no-default-features --features postgres
diesel setup
diesel migration run
```

This command will setup the PostgreSQL database.

## Building the service images

To build each service's Docker image, in the root directory, run:

```bash
docker build -f services/{service_name}/Dockerfile -t {your_username}/tklchat-{service_name}:0.1.0-build1 .
```

## Cleaning Up

To remove containers **and** their volumes completely:

```bash
docker compose -f "compose.yaml" down -v
```

## Tech Stack

### Languages

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)

### Frameworks & Libraries

![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=%23FFFFFF)
![Next JS](https://img.shields.io/badge/Next-black?style=for-the-badge&logo=next.js&logoColor=white)

### Tooling

![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![Obsidian](https://img.shields.io/badge/Obsidian-%23483699.svg?style=for-the-badge&logo=obsidian&logoColor=white)

## Contributing

We welcome contributions! To get started:

1. Fork the repo
2. Create a new branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

## License

GPL-3.0 — see [`LICENSE`](./LICENSE) for details.
