---
description: >-
  Our official Rust SDK is here and available on crates.io
featured: false
tags: [engineering]
image: ".gitbook/assets/image (2) (2).png"
---

# Announcing the Release of our Rust SDK

<div align="left">

<figure><img src=".gitbook/assets/silas.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Silas Marvin

June 4, 2024

We are excited to announce the official release of our Rust SDK for PostgresML, now available on [crates.io](https://crates.io/crates/pgml).

```bash
cargo add pgml
```

For those who have been with us for a while, you may already know that our Rust SDK has been a core component of our development. Our JavaScript, Python, and C SDKs are actually thin wrappers around our Rust SDK. We previously detailed this process in our blog post [How We Generate JavaScript and Python SDKs From Our Canonical Rust SDK](https://postgresml.org/blog/how-we-generate-javascript-and-python-sdks-from-our-canonical-rust-sdk).

Although our Rust SDK has been available on GitHub for some time, this marks its official debut on [crates.io](https://crates.io/crates/pgml). Alongside this release, we've also introduced [rust_bridge](https://crates.io/crates/rust_bridge), the crate we utilize to generate our JavaScript, Python, and now C SDKs from our Rust base.

Thank you for your continued support as we innovate in building multi-language SDKs with feature parity.
