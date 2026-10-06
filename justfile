set shell := ["nu", "-c"]
set dotenv-load

# list recipes
default:
  @just --list

# run an OpenTofu command
[working-directory: "./infra/"]
tofu *args:
  ../tools/run-tofu.nu {{ args }}

# run a SOPS command
sops *args:
  sops --config ./infra/vars/.sops.yaml {{ args }}
