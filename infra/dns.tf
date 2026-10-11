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

resource "cloudflare_dns_record" "umami_a" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "umami"
  type    = "A"
  content = var.vps_ipv4
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "umami_aaaa" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "umami"
  type    = "AAAA"
  content = var.vps_ipv6
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "apex_txt_sl_verification" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "@"
  type    = "TXT"
  content = "sl-verification=fxncjuftchgummqcrwzrpmyucobgvg"
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "apex_mx" {
  for_each = {
    route1 = {
      value    = "mx1.simplelogin.co."
      priority = 10
    }
    route2 = {
      value    = "mx2.simplelogin.co."
      priority = 20
    }
  }

  zone_id  = data.cloudflare_zone.acearchive.id
  name     = "@"
  type     = "MX"
  content  = each.value.value
  priority = each.value.priority
  ttl      = local.auto_ttl
  proxied  = false
}

resource "cloudflare_dns_record" "apex_txt_spf" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "@"
  type    = "TXT"
  content = "v=spf1 include:simplelogin.co ~all"
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "apex_cname_dkim" {
  for_each = {
    record1 = {
      name  = "dkim._domainkey"
      value = "dkim._domainkey.simplelogin.co."
    }

    record2 = {
      name  = "dkim02._domainkey"
      value = "dkim02._domainkey.simplelogin.co."
    }

    record3 = {
      name  = "dkim03._domainkey"
      value = "dkim03._domainkey.simplelogin.co."
    }
  }

  zone_id = data.cloudflare_zone.acearchive.id
  name    = each.value.name
  type    = "CNAME"
  content = each.value.value
  ttl     = local.auto_ttl
  proxied = false
}

resource "cloudflare_dns_record" "apex_txt_dmarc" {
  zone_id = data.cloudflare_zone.acearchive.id
  name    = "_dmarc"
  type    = "TXT"
  content = "v=DMARC1; p=quarantine; pct=100; adkim=s; aspf=s"
  ttl     = local.auto_ttl
  proxied = false
}
