module "petty_matters_app" {
  source = "./modules/app"

  label        = "production"
  region       = "fra1"
  service_name = "petty-matters-forum"
  domain_name  = "ministryofpettymatters.com"

  instance_count     = 1
  instance_size_slug = "apps-s-1vcpu-0.5gb"

  registry_type     = "DOCR"
  docker_image_name = "ministry"
  docker_image_tag  = "latest"
}