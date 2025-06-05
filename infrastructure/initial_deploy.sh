# assumes docker, doctl, and tofu are installed and configured
tofu apply -target=digitalocean_container_registry.container_registry -auto-approve
cd .. && docker buildx build --platform=linux/amd64 -t registry.digitalocean.com/petty-matters/ministry:latest .
cd migration docker buildx build --platform=linux/amd64 -t registry.digitalocean.com/petty-matters/ministry:deps .
docker push registry.digitalocean.com/petty-matters/ministry:latest
docker push registry.digitalocean.com/petty-matters/ministry:deps
tofu apply -auto-approve