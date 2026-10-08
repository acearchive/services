set shell := ["nu", "-c"]
set dotenv-load

# list recipes
default:
  @just --list

# run the media proxy locally
[group("media proxy")]
[working-directory: "./media-proxy/"]
run-media-proxy:
  npx wrangler@latest dev

# deploy the media proxy
[group("media proxy")]
[working-directory: "./media-proxy/"]
deploy-media-proxy:
  npx wrangler@latest deploy

# run the hha archive site locally
[group("hha archive")]
[working-directory: "./hha-archive/"]
run-hha-archive:
  npx wrangler@latest dev

# deploy the hha archive site
[group("hha archive")]
[working-directory: "./hha-archive/"]
deploy-hha-archive:
  npx wrangler@latest deploy

# run an opentofu command
[group("infrastructure")]
tofu *args:
  ./tools/tofu.nu {{ args }}

# run a sops command
[group("infrastructure")]
sops *args:
  sops --config ./infra/vars/.sops.yaml {{ args }}
