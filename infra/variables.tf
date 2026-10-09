variable "cloudflare_api_token" {
  type        = string
  description = "Cloudflare API token"
  sensitive   = true
}

variable "cloudflare_account_id" {
  type        = string
  description = "Cloudflare account ID"
}

variable "vps_ipv4" {
  type        = string
  description = "IPv4 address of the VPS"
}

variable "vps_ipv6" {
  type        = string
  description = "IPv6 address of the VPS"
}
