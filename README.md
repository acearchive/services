# Ace Archive Services

This is the monorepo for the various backend services that support [Ace
Archive](https://acearchive.lgbt).

- [Omeka Data Connector](./omeka-connector/): This service is deployed
  alongside the [Omeka S
  installation](https://github.com/acearchive/omeka-deployment) and sits in front
  of the Omeka S API. It has a few responsibilities, detailed in its own README.
- [Media Proxy](./media-proxy/): This edge function is a small reverse proxy
  that sits between users and the Omeka Data Connector. Its job is to rewrite
  URLs and facilitate edge caching of media files.
- [Haven for the Human Amoeba Archive](./hha-archive/): This static site serves
  a browsable archive of the now-defunct [Haven for the Human
  Amoeba](https://acearchive.lgbt/artifacts/haven-for-the-human-amoeba/) Yahoo
  Group.
- [Infrastructure](./infra/): This OpenTofu configuration declaratively
  describes the project's cloud infrastructure.
