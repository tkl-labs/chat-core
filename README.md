# TKL Chat: a cross-platform chat room

This repository contains the core of TKL Chat, a cross-platform chat room.

## Getting Started

### Creating Docker containers and volumes

```bash
docker compose -f "compose.yaml" up -d
```

### Initialising the database using diesel

```bash
cargo install diesel_cli --no-default-features --features postgres
# important: at this point, you should have defined the DATABASE_URL variable in `.env`
diesel setup
diesel migration run
```

### Running the server

```bash
cargo run --bin server
```

### Completely removing Docker containers and related volumes

```bash
docker compose -f "compose.yaml" down -v
```

## Tech Stack

### Languages:

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)

### Frameworks:

![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=%23FFFFFF)
![Next JS](https://img.shields.io/badge/Next-black?style=for-the-badge&logo=next.js&logoColor=white)

### Miscallaneous:

![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![Obsidian](https://img.shields.io/badge/Obsidian-%23483699.svg?style=for-the-badge&logo=obsidian&logoColor=white)
