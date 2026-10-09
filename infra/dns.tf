locals {
  # Cloudflare interprets this TTL to mean "automatic".
  auto_ttl = 1
}

resource "cloudflare_dns_record" "omeka_a" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "omeka"
  type    = "A"
  content = var.vps_ipv4
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "omeka_aaaa" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "omeka"
  type    = "AAAA"
  content = var.vps_ipv6
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "connector_a" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "connector"
  type    = "A"
  content = var.vps_ipv4
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "connector_aaaa" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "connector"
  type    = "AAAA"
  content = var.vps_ipv6
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_record" "umami_cname" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "umami"
  type    = "CNAME"
  content = "truthful-silkworm.pikapod.net"
  ttl     = local.auto_ttl
  proxied = false
}
