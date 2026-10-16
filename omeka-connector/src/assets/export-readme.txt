What is this?
=============

This is a bulk data export from Ace Archive, a digital archive of queer history
with a particular focus on the history of the asexual and aromantic
communities.

https://acearchive.lgbt

This data export contains the full Ace Archive collection—data and metadata.
This export is designed to be both human- and machine-readable; you can parse
the data with a computer program, import it into another system, or just
browse.

This data was exported on: {{ timestamp }}

What's in this folder?
======================

In this folder you'll find a file named `metadata.jsonld`, which contains the
metadata for the entire collection in JSON-LD format. This folder also contains
a subfolder named `files`, which contains all of the files in the collection,
organized by which item they're associated with.

How do I read the metadata?
===========================

The metadata in `metadata.jsonld` is in JSON-LD format. JSON-LD is a computer
language for storing data. You can open this file in a text editor and read it
yourself, but it is primarily intended to be read by a computer program. The
following sections will explain the technical details of how to read this data.

What is JSON-LD?
================

JSON-LD is a JSON-based format for representing directed graphs of linked data.
It is an implementation of the Resource Description Framework (RDF), which is a
standard model for data interchange on the web.

https://json-ld.org/

https://www.w3.org/RDF/

How does this project use JSON-LD?
==================================

The metadata in this data export uses the Schema.org vocabulary to describe
items in the collection and how they're related.

https://schema.org/

This project extends the Schema.org vocabulary with two additional terms. Note
that these URLs are identifiers, not links to web pages.

    `https://acearchive.lgbt/terms/slug` (`ace:slug`)
    `https://acearchive.lgbt/terms/filename` (`ace:filename`)

These terms are how you map metadata in `metadata.jsonld` to files in the
`files` directory. The files directory has the following format.

    files/{ace:slug}/{ace:filename}

For example, if an item in the collection has an `ace:slug` value of
`orlando-the-asexual-manifesto` and references a file with an `ace:filename`
value of `the-asexual-manifesto.pdf`, you would find that file in the following
location:

    files/orlando-the-asexual-manifesto/the-asexual-manifesto.pdf

How can I parse this metadata?
==============================

Many programming languages have libraries available for parsing JSON-LD. You
can also use a general-purpose JSON parser, but be aware that two JSON-LD
documents can be semantically identical but have very different structures in
JSON.

The structure of this metadata may change without warning between versions of
this data export. If the export contains a file named `metadata.jsonld`, it
will always be in JSON-LD format. However, the specific vocabularies and
properties used in the metadata are not guaranteed to remain the same.
