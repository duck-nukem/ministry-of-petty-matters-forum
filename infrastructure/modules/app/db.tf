resource "digitalocean_database_cluster" "main" {
  name       = "${var.label}-db"
  engine     = "pg"
  version    = "17"
  size       = "db-s-1vcpu-1gb"
  region     = var.region
  node_count = 1
}

resource "digitalocean_database_firewall" "db_firewall" {
  cluster_id = digitalocean_database_cluster.main.id

  rule {
    type  = "app"
    value = digitalocean_app.app_platform.id
  }
}

output "db_connection_string" {
  value = digitalocean_database_cluster.main.uri
}