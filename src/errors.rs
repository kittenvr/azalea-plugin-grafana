[package]
name = "azalea_plugin_grafana"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A Minecraft server monitoring plugin for Azalea"

[dependencies]
azalea = "0.1"
prometheus = "0.13"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
warp = "0.3"
toml = "0.7"
thiserror = "1.0"
async-trait = "0.1"
tracing = "0.1"
