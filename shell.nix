{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy

    # SQLite
    sqlite

    # SQLx CLI for migrations
    sqlx-cli

    # Python with bcrypt for user creation script
    (python3.withPackages (ps: with ps; [
      bcrypt
    ]))

    # Development tools
    pkg-config
    openssl

    # Docker tools (optional)
    docker
    docker-compose
  ];

  shellHook = ''
    export DATABASE_URL="sqlite:scrob.db"
    export RUST_LOG="scrob=debug,tower_http=debug"

    echo "Scrob development environment loaded"
    echo ""
    echo "Available commands:"
    echo "  cargo build          - Build the project"
    echo "  cargo run            - Run the server"
    echo "  cargo test           - Run tests"
    echo "  cargo sqlx migrate run - Run database migrations"
    echo "  ./scripts/create_user.sh <user> <pass> [admin] - Create user"
    echo ""
    echo "Server will run on http://127.0.0.1:3000"
    echo "GraphQL Playground: http://127.0.0.1:3000/playground"
  '';
}
