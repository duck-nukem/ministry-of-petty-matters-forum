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

resource "digitalocean_database_user" "app_user" {
  cluster_id = digitalocean_database_cluster.main.id
  name       = "app_user"
}

resource "digitalocean_database_connection_pool" "connection_pool" {
  cluster_id = digitalocean_database_cluster.main.id

  name    = "app_pool"
  db_name = "defaultdb"
  user    = "doadmin"  # use digitalocean_database_user.app_user

  mode = "transaction"
  size = 21
}

output "db_connection_string" {
  value = digitalocean_database_connection_pool.connection_pool.private_uri
}