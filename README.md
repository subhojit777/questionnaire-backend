# Questionnaire

A real time crowd survey application.

This repository makes the backend of the application. It provides REST API endpoints which you can access via any REST client.

## Preqrequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [MySQL](https://dev.mysql.com/downloads/mysql)

## Installation

```
git clone https://github.com/subhojit777/questionnaire-rs.git
cd questionnaire-rs
cp .env.example .env
# Replace the values in .env as necessary
cargo install diesel_cli
diesel migration run
cargo build
```

## Start the server

```
cargo run
```
