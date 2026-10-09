data "cloudflare_zone" "acearchive" {
  filter = {
    name = "acearchive.lgbt"
    account = {
      id = var.cloudflare_account_id
    }
  }
}

resource "cloudflare_zone_dnssec" "acearchive" {
  zone_id = data.cloudflare_zone.acearchive.id
}

resource "cloudflare_zone_setting" "always_use_https" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "always_use_https"
  value      = "on"
}

resource "cloudflare_zone_setting" "automatic_https_rewrites" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "automatic_https_rewrites"
  value      = "on"
}

resource "cloudflare_zone_setting" "min_tls_version" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "min_tls_version"
  value      = "1.2"
}

resource "cloudflare_zone_setting" "tls_1_3" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "tls_1_3"
  value      = "on"
}

resource "cloudflare_zone_setting" "ssl" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "ssl"
  value      = "strict"
}

resource "cloudflare_zone_setting" "http3" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "http3"
  value      = "on"
}

resource "cloudflare_zone_setting" "brotli" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "brotli"
  value      = "on"
}

resource "cloudflare_zone_setting" "early_hints" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "early_hints"
  value      = "on"
}

resource "cloudflare_zone_setting" "webmcp_enabled" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "webmcp_enabled"
  value      = "off"
}

resource "cloudflare_zone_setting" "ipv6" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "ipv6"
  value      = "on"
}

resource "cloudflare_zone_setting" "email_obfuscation" {
  zone_id    = data.cloudflare_zone.acearchive.id
  setting_id = "email_obfuscation"
  value      = "on"
}
