terraform {
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.27.0"
    }
  }

  backend "s3" {
    key    = "tofu.tfstate"
    bucket = "acearchive-tofu"
    endpoints = {
      s3 = "https://dd48e1a60e3648ec3f391663eadf89b8.r2.cloudflarestorage.com"
    }
    region                      = "auto"
    use_lockfile                = true
    skip_credentials_validation = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    skip_metadata_api_check     = true
  }
}

provider "cloudflare" {
  api_token = var.cloudflare_api_token
}
