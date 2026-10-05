# Omeka Data Connector

This service is deployed alongside the Omeka S installation and sits between
the frontend edge functions and the Omeka S API. It has a few responsibilities:

1. Serve media files from the collection.
2. Serve metadata that the [Hugo
   site](https://github.com/acearchive/acearchive.lgbt) ingests at build time.
3. Generate periodic backups of the collection in portable human- and
   machine-readable formats suitable for distribution.
