# Media Proxy

This edge function is a small reverse proxy that sits between users and the
Omeka Data Connector. Its job is to rewrite URLs and facilitate edge caching of
media files.

This edge function is written on the [Cloudflare
Workers](https://developers.cloudflare.com/workers/) platform.

## What it does

This edge function forwards requests to `files.acearchive.lgbt` to the Omeka
Data Connector. It accepts URLs that follow one of these patterns.

```
https://files.acearchive.lgbt/artifacts/<ARTIFACT_SLUG>/<FILENAME>
https://files.acearchive.lgbt/a/<ARTIFACT_ID>/<FILENAME>
https://files.acearchive.lgbt/r/<ARTIFACT_ID>/<FILENAME>
```

The following are examples of actual file URLs.

```
https://files.acearchive.lgbt/artifacts/choose-your-label-barnard-college/choose-your-label.png
https://files.acearchive.lgbt/a/cmD6HyZzoISB/choose-your-label.png
https://files.acearchive.lgbt/r/cmD6HyZzoISB/choose-your-label.png
```

Omeka S and the Omeka Data Connector run on a single server in a single region.
Scaling a VPS is expensive. This edge function caches media on the Cloudflare
network to reduce load on the Omeka server.

## Deployment

Building and deploying this edge function requires the following tools:

- [npm](https://nodejs.org/en/download)
- [Cargo](https://rust-lang.org/tools/install/)
