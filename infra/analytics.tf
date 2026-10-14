# Disable web analytics.
resource "cloudflare_web_analytics_site" "acearchive" {
  account_id   = var.cloudflare_account_id
  auto_install = false
  enabled      = false
  zone_tag     = data.cloudflare_zone.acearchive.id
}
