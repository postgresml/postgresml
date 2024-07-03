---
description: Korvus is an SDK for JavaScript, Python and Rust implements common use cases and PostgresML connection management.
---

# Korvus

Korvus can be installed using standard package managers for JavaScript, Python, and Rust. Since the SDK is written in Rust, the JavaScript and Python packages come with no additional dependencies.

For key features, a quick start, and the code see [the Korvus GitHub](https://github.com/postgresml/korvus)

Common links:
- [API docs](api/)
- [Guides](guides/)
- [Example Apps](example-apps/)

## Installation

Installing the SDK into your project is as simple as:

{% tabs %}
{% tab title="JavaScript" %}
```bash
npm i korvus
```
{% endtab %}

{% tab title="Python" %}
```bash
pip install korvus
```
{% endtab %}

{% tab title="Rust" %}
```bash
cargo add korvus
```
{% endtab %}

{% tab title="C" %}

First clone the `korvus` repository and navigate to the `korvus/c` directory:
```bash
git clone https://github.com/postgresml/korvus
cd korvus/korvus/c
```

Then build the bindings
```bash
make bindings
```

This will generate the `korvus.h` file and a `.so` on linux and `.dyblib` on MacOS.
{% endtab %}
{% endtabs %}

## Connect to PostgresML

The SDK automatically manages connections to PostgresML. The connection string can be specified as an argument to the collection constructor, or as an environment variable.

If your app follows the twelve-factor convention, we recommend you configure the connection in the environment using the `KORVUS_DATABASE_URL` variable:

```bash
export KORVUS_DATABASE_URL=postgres://user:password@sql.cloud.postgresml.org:6432/korvus_database
```

## Next Steps

Common links:
- [API docs](api/)
- [Guides](guides/)
- [Example Apps](example-apps/)
