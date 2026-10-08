# Haven for the Human Amoeba Archive

This static site serves a browsable archive of the now-defunct [Haven for the
Human Amoeba](https://acearchive.lgbt/artifacts/haven-for-the-human-amoeba/)
Yahoo Group.

This site is set up to deploy on the [Cloudflare
Workers](https://developers.cloudflare.com/workers/) platform. Building the
site downloads the HHA archive from Ace Archive (if not already present
locally), unzips it, and generates a static site using
[acearchive/yahoo-groups-reader](https://github.com/acearchive/yahoo-groups-reader),
which is included here as a git submodule.

## Deployment

Building and deploying this site requires the following tools:

- `unzip`
- [Nushell](https://www.nushell.sh/book/installation.html)
- [Go](https://go.dev/dl/)
- [npm](https://nodejs.org/en/download)
