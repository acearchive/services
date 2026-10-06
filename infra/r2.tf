locals {
  backup_retention = {
    daily   = { lock_days = 7, expire_days = 8 }
    weekly  = { lock_days = 28, expire_days = 35 }
    monthly = { lock_days = 365, expire_days = 395 }
  }

  day_seconds = 24 * 60 * 60
}

resource "cloudflare_r2_bucket" "omeka_backups" {
  account_id = var.cloudflare_account_id
  name       = "acearchive-omeka-backups"
}

# Bucket lifecycle rules automatically prune old backups.
resource "cloudflare_r2_bucket_lifecycle" "backups" {
  account_id  = var.cloudflare_account_id
  bucket_name = cloudflare_r2_bucket.backups.name

  rules = [for tier, retention in local.backup_retention : {
    id         = "expire-${tier}"
    enabled    = true
    conditions = { prefix = "${tier}/" }
    delete_objects_transition = {
      condition = {
        type    = "Age"
        max_age = retention.expire_days * local.day_seconds
      }
    }
  }]
}

# Bucket locks prevent backups from being overwritten or deleted prematurely.
resource "cloudflare_r2_bucket_lock" "backups" {
  account_id  = var.cloudflare_account_id
  bucket_name = cloudflare_r2_bucket.backups.name

  rules = [for tier, retention in local.backup_retention : {
    id      = "lock-${tier}"
    enabled = true
    prefix  = "${tier}/"
    condition = {
      type            = "Age"
      max_age_seconds = retention.lock_days * local.day_seconds
    }
  }]
}
