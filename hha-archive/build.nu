#!/usr/bin/env nu

const HHA_ARCHIVE_ZIP_URL = "https://files.acearchive.lgbt/a/PZsGT3uJ1FXk/haven-for-the-human-amoeba.zip"

def main [] {
  let current_dir = $env.FILE_PWD
  let builder_repo = $current_dir | path join "yahoo-groups-reader"
  let zip_path = $current_dir | path join "archive.zip"
  let extract_path = $current_dir | path join "archive"
  let output_path = $current_dir | path join "output"
  let public_path = $current_dir | path join "public"
  let email_path = $extract_path | path join "email"

  rm --recursive --force $zip_path $extract_path $output_path $public_path

  http get --raw $HHA_ARCHIVE_ZIP_URL | save $zip_path
  unzip $zip_path -d $extract_path

  cd ($builder_repo | path join "parser")
  go run . $email_path --output $output_path --title "Haven for the Human Amoeba" --base "https://hha.acearchive.lgbt/" --link "archive,Ace Archive,https://acearchive.lgbt/artifact/haven-for-the-human-amoeba/" --locale "en_US"

  let pipeline_vars = {
    OUTPUT_DIR: $output_path,
    PUBLIC_DIR: $public_path,
  }

  cd ($builder_repo | path join "pipeline")
  with-env $pipeline_vars {
    npm install
    npx gulp
  }
}
