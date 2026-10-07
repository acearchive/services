# Omeka Data Connector

This service is deployed alongside the [Omeka S
installation](https://github.com/acearchive/omeka-deployment) and sits in front
of the Omeka S API. It has a few responsibilities:

1. Serve media files from the collection.
2. Serve metadata that the [Hugo
   site](https://github.com/acearchive/acearchive.lgbt) ingests at build time.
3. Generate periodic data exports of the collection in portable human- and
   machine-readable formats suitable for distribution.

## What it does

### Serving files

This service handles mapping Ace Archive artifact IDs, artifact slugs, and
filenames to the corresponding media file URLs in Omeka.

Given one of the following URL patterns, it will find and reverse proxy to the
corresponding file in Omeka.

```
/media/artifacts/<ARTIFACT_SLUG>/<FILENAME>
/media/a/<ARTIFACT_ID>/<FILENAME>
/media/r/<ARTIFACT_ID>/<FILENAME>
```

This service is intended to be deployed behind another reverse proxy so it can
serve files from `files.acearchive.lgbt` instead of the domain Omkea is hosted
under. The following are examples of actual file URLs.

```
https://files.acearchive.lgbt/artifacts/choose-your-label-barnard-college/choose-your-label.png
https://files.acearchive.lgbt/a/cmD6HyZzoISB/choose-your-label.png
https://files.acearchive.lgbt/r/cmD6HyZzoISB/choose-your-label.png
```

The URL mappings are cached in memory to avoid unnecessary lookups, but the
files themselves are always hosted directly from Omkea.

### Serving metadata

This service serves metadata about every item, item set, and media file in the
collection from `GET /items`. This metadata is in a format intended to be
consumed by [Hugo content
adapters](https://gohugo.io/content-management/content-adapters/) and used to
build the [static site](https://github.com/acearchive/acearchive.lgbt).

This single endpoint returns the entire collection without pagination. This
makes for a long response time (on the order of several seconds), but that is
acceptable because it is only called once at site build time and removes
significant complexity from the static site template.

### Generating data exports

This service generates periodic data exports of the collection in portable
human- and machine-readable formats suitable for distribution.

Each export builds a `.zip` file and pushes it to an S3 bucket for interested
parties to download.

## Configuration

This service expects the following environment variables, which it will read
from a `.env` file if one is available.

- `BASE_DOMAIN`: The domain of the static site.
- `OMEKA_URL`: The base URL of the Omeka instance, without the `/api/` path.
- `FILES_URL`: The base URL that files are served from. If a reverse proxy sits
  in front of this service, this should point to that proxy.
- `PORT`: The port to listen on.
- `SCRATCH_DIR`: A scratch directory for writing temporary data to. This does
  not need to be durable, but it is used for assembling large ZIP archives, so it
  should probably not point to a directory on tmpfs for similar.
- `DATA_EXPORT_CRON_SPEC`: A cron spec which determines the frequency of
  automatic data exports. If unset, automatic data exports are disabled.

## Deployment

A CI workflow publishes a container image `ghcr.io/acearchive/omeka-connector`.
The image includes a healthcheck binary at `/usr/local/bin/healthcheck` which
checks the server's `/health` endpoint.
