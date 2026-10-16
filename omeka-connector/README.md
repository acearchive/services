# Omeka Data Connector

This service is deployed alongside the [Omeka S
installation](https://github.com/acearchive/omeka-deployment) and sits in front
of the Omeka S API. It has a few responsibilities:

1. Serve media files from the collection.
2. Serve metadata that the [static site](https://github.com/acearchive/site)
   ingests at build time.
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

From top to bottom, these patterns are referred to as the "long" form, "short"
form, and "raw" form. The "short" URL redirects to the "long" one.

When using the "long" form, the file is embedded in an HTML document which
shows three buttons at the bottom of the viewport:

1. A link back to the corresponding page on Ace Archive.
2. A link to the "raw" URL.
3. A button to copy the "short" URL to the clipboard.

When using the "raw" form, the raw file is served instead.

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
collection from two endpoints: `GET /hugo-items` and `GET /hugo-metadata`.
These endpoints return metadata is a format intended to be consumed by [Hugo
content adapters](https://gohugo.io/content-management/content-adapters/) and
used to build the [static site](https://github.com/acearchive/site). These
endpoints **are not** public API.

The `GET /hugo-items` endpoint returns the entire collection without
pagination. This makes for a long response time (on the order of several
seconds), but that is acceptable because it is only called once at site build
time and removes significant complexity from the static site template.

### Generating data exports

This service generates periodic data exports of the collection in portable
human- and machine-readable formats suitable for distribution.

Each export builds a `.zip` file and pushes it to an S3 bucket for interested
parties to download. The archive has this layout:

```
README.txt
metadata.jsonld
files/{slug}/{filename}
```

`metadata.jsonld` contains the full JSON-LD graph of the collection, including
the project-specific `ace:slug` and `ace:filename` terms, which can be used to
locate each file under `files/`. The contents of `README.txt` live in
[`src/assets/export-readme.txt`](./src/assets/export-readme.txt).

Archives are uploaded to the bucket under these keys, with timestamps in UTC:

```
daily/%Y-%m-%dT%H:%M:%S.zip
weekly/%Y-%m-%dT%H:%M:%S.zip
monthly/%Y-%m-%dT%H:%M:%S.zip
latest.zip
```

Exports are tiered the same way as the [Omeka
backups](https://github.com/acearchive/omeka-deployment): an export is
`monthly` if there hasn't been a monthly export yet this month, `weekly` if
there hasn't been a monthly or weekly export yet this week (starting Monday),
and `daily` otherwise. Bucket lifecycle rules prune old exports. `latest.zip`
is a copy of the most recent export.

After each export, this service pings a [Healthchecks.io](https://healthchecks.io)
endpoint, if one is configured, so we are notified if exports stop happening.
It pings `/start` when an export begins and `/fail` when one fails.

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
- `MEDIA_CACHE_TTL_SECS`: How long, in seconds, to cache URL mappings in
  memory.
- `MODEL_CACHE_TTL_SECS`: How long, in seconds, to cache metadata about the
  collection in memory. This should be long enough to last through a build of
  the static site.
- `DATA_EXPORT_CRON_SPEC`: A cron spec which determines the frequency of
  automatic data exports, evaluated in UTC. If unset, automatic data exports
  are disabled, and the other `DATA_EXPORT_*` variables are ignored.
- `DATA_EXPORT_BUCKET`: The name of the S3 bucket to upload data exports to.
- `DATA_EXPORT_S3_ENDPOINT`: The S3 API endpoint, such as
  `https://<ACCOUNT_ID>.r2.cloudflarestorage.com` for Cloudflare R2.
- `DATA_EXPORT_S3_REGION`: The S3 region. Defaults to `auto`, which is what R2
  expects.
- `DATA_EXPORT_S3_ACCESS_KEY_ID`: The S3 access key ID.
- `DATA_EXPORT_S3_SECRET_ACCESS_KEY`: The S3 secret access key.
- `DATA_EXPORT_HEALTHCHECK_PING_URL`: An optional
  [Healthchecks.io](https://healthchecks.io) ping URL.

## Deployment

A CI workflow publishes a container image to
`ghcr.io/acearchive/omeka-connector` from `main`. The image includes a
healthcheck binary at `/usr/local/bin/healthcheck` which checks the server's
`/health` endpoint.
