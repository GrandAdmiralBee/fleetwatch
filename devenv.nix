{
  pkgs,
  config,
  ...
}:

{
  # --- Rust ---
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  # --- Нативные зависимости ---
  packages = with pkgs; [
    git
    protobuf
    pkg-config
    openssl
    sqlx-cli
    cargo-nextest
    cargo-deny
    mdbook
    mdbook-mermaid
    buf
  ];

  # --- Сервисы ---
  services.postgres = {
    enable = true;
    listen_addresses = "127.0.0.1";
    port = 5432;
    initialDatabases = [ { name = "fleetwatch"; } ];
  };

  services.redis.enable = true;

  # --- Переменные окружения ---
  env = {
    DATABASE_URL = "postgres://127.0.0.1:5432/fleetwatch";
    REDIS_URL = "redis://127.0.0.1:6379";
    RUST_LOG = "info,fleetwatch=debug";
  };

  # --- Dev Команды ---
  scripts.db-migrate.exec = "sqlx migrate run";
  scripts.db-prepare.exec = "cargo sqlx prepare --workspace";

  # --- Git Hooks ---
  git-hooks.hooks = {
    nixfmt.enable = true;
    rustfmt.enable = true;
    clippy.enable = true;
  };

  enterShell = ''
    echo "fleetwatch dev shell: rustc $(rustc --version | cut -d' ' -f2)"
  '';
}
