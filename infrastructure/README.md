# Deployment

## Known issues

If you start from scratch, you have to:

1. run tofu apply to create the DOCR registry
2. tofu fails, because there are no images pushed
3. build and push an image to the registry
4. run tofu apply again