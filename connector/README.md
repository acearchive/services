# Ace Archive Data Connector

This service sits alongside the Omeka S installation and is responsible for
pulling data out of it. It has a few responsibilities:

1. Build [Hugo artifacts](https://github.com/acearchive/hugo-artifacts/) and
   push them to a git repo that Hugo ingests to build the static site.
2. Push uploaded files to the S3 bucket where they're served for the static
   site.
3. Generate backups of the collection in portable human- and machine-readable
   formats suitable for distribution.
