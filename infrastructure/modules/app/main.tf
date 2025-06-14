resource "digitalocean_app" "app_platform" {
  spec {
    name   = var.label
    region = var.region

    domain {
      name = var.label == "production" ? var.domain_name : "${var.label}.${var.domain_name}"
      type = "PRIMARY"
      zone = var.domain_name
    }

    job {
      name = "migrate"

      kind        = "PRE_DEPLOY"
      run_command = "migration up -u ${digitalocean_database_cluster.main.uri}"

      env {
        key   = "DATABASE_URL"
        value = digitalocean_database_cluster.main.uri

        scope = "RUN_TIME"
        type  = "SECRET"
      }

      image {
        registry_type = var.registry_type
        repository    = var.task_runner_docker_image_name
        tag           = var.task_runner_docker_image_tag

        deploy_on_push {
          enabled = true
        }
      }
    }

    service {
      name               = var.service_name
      instance_count     = var.instance_count
      instance_size_slug = var.instance_size_slug

      http_port = 3000

      env {
        key   = "PUBLIC_ROOT_URL"
        value = "https://${var.label == "production" ? var.domain_name : "${var.label}.${var.domain_name}"}"
        scope = "RUN_TIME"
      }

      env {
        key   = "RUST_BACKTRACE"
        value = "1"
        scope = "RUN_TIME"
      }

      env {
        key   = "DATABASE_URL"
        value = digitalocean_database_connection_pool.connection_pool.uri
        scope = "RUN_TIME"
        type  = "SECRET"
      }

      image {
        registry_type = var.registry_type
        repository    = var.docker_image_name
        tag           = var.docker_image_tag

        deploy_on_push {
          enabled = true
        }
      }

      health_check {
        http_path = "/"
        port      = 3000
      }
    }
  }
}