# {{ item.title }}

## Abstract

{{ item.summary }}
{% if item.description is defined %}
## Description

{{ item.description }}
{% endif %}
## Date Created

{{ item.from_year }}{% if item.to_year is defined and item.to_year != item.from_year %} - {{ item.to_year }}{% endif %}
{% if item.people %}
## People
{% for person in item.people %}
- {{ person.name }}
{%- endfor %}
{% endif -%}
{% if item.identities %}
## Identities
{% for identity in item.identities %}
- {{ identity.name }}
{%- endfor %}
{% endif -%}
{% if item.collections %}
## Collections
{% for collection in item.collections %}
- {{ collection.title }}{% if collection.description is defined %}: {{ collection.description }}{% endif %}
{%- endfor %}
{% endif -%}
{% if item.links %}
## Links
{% for link in item.links %}
- [{{ link.title }}]({{ link.url }})
{%- endfor %}
{% endif -%}
{% if item.files %}
## Files
{% for file in item.files %}
- [{{ file.title }}](files/{{ file.filename }})
{%- endfor %}
{% endif -%}
