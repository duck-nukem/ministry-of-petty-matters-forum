resource "digitalocean_container_registry" "container_registry" {
  name                   = "petty-matters"
  subscription_tier_slug = "starter"
}
