# Infrastructure

Infrastructure for this project is managed with
[OpenTofu](https://opentofu.org/). Secrets for managing infrastructure are
encrypted with Frawley's SSH key via [age](age-encryption.org) and committed to
the repo.

Working with this project's infrastructure requires the following tools:

- [just](https://just.systems/man/en/installation.html)
- [Nushell](https://www.nushell.sh/book/installation.html)
- [OpenTofu](https://opentofu.org/docs/intro/install/)
- [SOPS](https://getsops.io/)
- [age](https://age-encryption.org/)

To run `tofu` commands against this repo, use `just tofu`. This wrapper
decrypts secrets and passes them to OpenTofu.

- Plaintext OpenTofu variables are defined in
  [vars/vars.yaml](./vars/vars.yaml).
- Secret OpenTofu variables are defined in
  [vars/secrets.enc.yaml](./vars/secrets.enc.yaml).
- Secret environment variables for configuring the OpenTofu backend are defined
  in [vars/env.enc.yaml](./vars/env.enc.yaml).

To deploy infrastructure, you'll first need your SSH key authorized by adding
it to [vars/.sops.yaml](./vars/.sops.yaml) and running these commands:

```
cd ./infra/vars/
just sops updatekeys ./secrets.enc.yaml
just sops updatekeys ./env.enc.yaml
```

Once your key is authorized, set the env var `SOPS_AGE_SSH_PRIVATE_KEY_FILE` to
the path of your private SSH key. You can put this in a `./.env` file in the
root of the repo; it will be ignored by git.

You can edit OpenTofu secrets interactively like this:

```
cd ./infra/vars/
just sops edit ./secrets.enc.yaml
```
