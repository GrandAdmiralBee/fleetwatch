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

  # --- Dev packages ---
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

  # --- Services ---
  services.postgres = {
    enable = true;
    listen_addresses = "127.0.0.1";
    port = 5432;
    initialDatabases = [ { name = "fleetwatch"; } ];
  };

  services.redis.enable = true;

  # --- ENV Variables ---
  env = {
    DATABASE_URL = "postgres://127.0.0.1:5432/fleetwatch";
    REDIS_URL = "redis://127.0.0.1:6379";
    RUST_LOG = "info,fleetwatch=debug";
  };

  # --- Dev Commands ---
  scripts.db-migrate.exec = "sqlx migrate run";
  scripts.db-prepare.exec = "cargo sqlx prepare --workspace";

  # --- Git Hooks ---
  git-hooks.hooks = {
    nixfmt.enable = true;
    rustfmt.enable = true;
    clippy.enable = true;

    prettier-docs = {
      enable = true;
      name = "prettier (docs)";
      entry = "${pkgs.prettier}/bin/prettier --write";
      files = "^docs/.*\\.md$";
      excludes = [ "^docs/book/" ];
      language = "system";
    };
  };

  enterShell = ''
    echo "fleetwatch dev shell: rustc $(rustc --version | cut -d' ' -f2)"
  '';
}
