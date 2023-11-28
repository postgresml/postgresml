# Packages

A collection of installable packages and libraries used for distributing and working with PostgresML.

## Table of contents

1. `cargo-pgml-components`: cargo CLI for building our web apps (e.g. `pgml-dashboard`)
2. `pgml-components`: library implementing common Rust components used in our web apps
3. `postgresml`: meta package used to install all PostgresML components on an Ubuntu system
4. `postgresml-dashboard`: web app for managing PostgresML, Ubuntu package
5. `postgresml-python`: Python dependencies shipped as a pre-built virtual environment, Ubuntu package
6. `postgresql-pgml`: PostgreSQL extension, Ubuntu package

## Packages

### `cargo-pgml-components`

A cargo (Rust build tool chain) CLI for building web apps written in Rust, Rocket, Sailfish and Turbo/Stimulus. It automatically creates the necessary folder structure for a project, and bundles JavaScript and Sass files into JS and CSS bundles. See [README.md](cargo-pgml-components/README.md) for more details.

### `pgml-components`

A Rust library implementing common components for web apps written in Rust, Rocket and Sailfish. Used in our web apps together with the cargo CLI.

### `postgresml`

A Debian (Ubuntu 22.04) package which installs everything needed for PostgresML to work on an Ubuntu system. It depends on `postgresql-pgml` and `postgresml-python`, ensuring that correct versions of both are installed, as needed.

### `postgresml-dashboard`

A Debian (Ubuntu 22.04) package which compiles and distributes `pgml-dashboard`. It follows the same release cadence as the extension package, documented below. The dashboard is distributed separately because it's a web app and often won't run on the same system as PostgresML.

### `postgresml-python`

A Debian (Ubuntu 22.04) package which builds and distributes a Python virtual environment with all the required Python packages used by PostgresML. This includes HuggingFace, PyTorch, Scikit-learn, XGBoost, and many more. This package is quite large and distributed separately since we update our PostgreSQL extension more frequently than our Python dependencies.

### `postgresql-pgml`

A Debian (Ubuntu 22.04) package which builds and distributes the PostgreSQL extension. The extension, better known as PostgresML, is the foundation of our product suite and performs all machine learning operations. It's distributed separately from Python dependencies because it includes many algorithms that don't require Python (i.e. our `rust` runtime). Additionally, some systems manage Python dependencies outside of a virtual environment, and we don't want to mandate its use.

## Dependency tree

![dependency tree](./dependency-tree.png)

## Release process

When releasing a new version of PostgresML to the community, we follow the following release process. At the moment, there are a few manual steps documented below. There are opportunities for automation which we haven't had a moment to explore yet, but PRs and ideas here are welcome.

### 1. Update version numbers

#### Cargo.toml

The version of PostgresML is set in many places, and all of them need to be updated. The first place is `Cargo.toml` (and `Cargo.lock`). Update it in `pgml-extension` and `pgml-dashboard`, making sure both of them match. This is helpful to our users because a version of the dashboard is guaranteed to work with the same version of the extension. The version in `pgml-extension/Cargo.toml` is automatically propagated to the PostgreSQL extension. Make sure to `cargo build` both packages to update the `Cargo.lock` lockfile as well.

#### Documentation

Additionally, we mention the version of the extension in our documentation. It would be very helpful to update it there as well, so our users are always instructed to install the latest and greatest version. Our documentation is located in `pgml-cms`. If you search it for the current version number, you should find all the places where we mention it.

#### Github Actions

We use Github actions to build our packages. The version of the Debian (Ubuntu) package is independently set from the extension version, but must match nonetheless to avoid confusion. The package version is passed into the build scripts using a Github Actions input variable. Currently, our build process is triggered manually, so that version should be either updated in the YAMLs, so the default is correct, or set manually by the release manager.

Currently, we have two Github Actions:

- `ubuntu-packages-and-docker-image`: builds `postgresml`, `postgresml-pgml`, `postgresml-dashboard`, and the Docker image
- `ubuntu-postgresml-python-package`: builds the `postgresml-python` package

The version of the `postgresml-python` should match the version of `postgresml` only when Python dependencies change.

### 2. Write a migration

PostgreSQL extensions are installed into stateful databases. Therefore, a migration from the previous version to the new one is always required. A migration changes the system in a way to work with the new version of the extension. This is especially needed when we change or add an API (SQL function). Without altering/adding the SQL function definitions, PostgreSQL won't be able to use the functions we changed in the extension code. If no API changes were made, PostgreSQL extensions still require a migration file, which can be empty. This is commonly done for bug fixes for existing functionality or internal changes, e.g. swapping runtimes or adding additional options already supported by the existing kwarg-based API.

#### How to write a migration

A migration is a SQL (`.sql`) text file that has to be placed into the `pgml-extension/sql` folder. The name of the file is following a strict convention that needs to be followed for the migration to work:

```
<name>--<current version>--<new version>.sql
```

where:

- `<name>` is the name of the extension, in our case, `pgml`
- `<current version>` is the currently released version of the extension, including minor and patch version numbers
- `<new version>` is the new version of the extension being released with the PR, including minor and patch version numbers

For example, if the current version of the extension is `2.7.10` and we would like to release the new version `2.7.11`, the migration file should be: `pgml-extension/sql/pgml--2.7.10--2.7.11.sql`.

When the extension is packaged, the migration will be included automatically by `pgrx`, our Rust PostgreSQL extension toolkit.

### 3. Commit and tag

Push a commit to `master` with the changes above. Once pushed, make a new Github release. The release should be named after the version, e.g. `v2.7.11` so we and our users can more easily find the changelog.

#### Name of the tag

Make sure that the tag is named correctly. Tags are immutable, so if we push the wrong name, it'll stay in git forever. Tags should be named with the version number preceded by the letter "v", e.g. `v2.7.11`.

### 4. Run Github Actions

In this order, run the Github actions:

1. If Python dependencies were updated, run `ubuntu-postgresml-python-package` and wait for it to complete
2. Run `ubuntu-packages-and-docker-image`. When that's finished, the new version of the extension will be released to everyone in our community.

