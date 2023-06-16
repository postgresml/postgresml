
## [devenv](https://deven.sh/) can be use to create an isolated development environment.

Here are a set of instructions to setup a development environment that is more interactive than
Docker containers while being isolated. It requires to install a Nix system on top of your current
Linux environment.


### Step 1: Install `devenv`

Follow the instructions from [devenv - Getting Started](https://deven.shgetting-started//):

- Install a base Nix system:
        `sh <(curl -L https://nixos.org/nix/install) --daemon`

- Install _Cachix_ which provides pre-compiled binaries (especially useful when installing
  Python packages):
        `nix-env -iA cachix -f https://cachix.org/api/v1/install`
        `cachix use devenv`
        `cachix use nixpkgs-python`

- Install `devenv`:
        `nix-env -if https://github.com/cachix/devenv/tarball/latest`

### Step 2: Prepare the environment

- We will use a local home for all Cargo crates: `mkdir .cargo_home`

- Within the top directory of the PostgresML repo, run `devenv init`.

- Create or replace a file named `devenv.nix` containing

```shell
{ pkgs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [
    pkgs.git
    pkgs.postgresql_15

    pkgs.bison
    pkgs.flex
    pkgs.cmake
    pkgs.gcc

    pkgs.openssl

    pkgs.llvmPackages.openmp
    pkgs.lightgbm
    pkgs.xgboost

    # Deepspeed requires either one.
    pkgs.cudaPackages.cuda_nvcc
    # pkgs.hipcc
    ];

    # https://devenv.sh/scripts/
    scripts.hello.exec = "\necho Starting with $GREET\n";

    enterShell = ''
    hello
    '';

    # https://devenv.sh/languages/
    languages.nix.enable = true;

    languages.python = {
        enable = true;
        version = "3.10";

        venv.enable = true;

        # Copied from requirements.txt
        # Removed because unneeded
        #   accelerate==0.19.0
        #   sacremoses==0.0.53
        #   einops==0.6.1

        # Breaks the install and does not seem required
        #   deepspeed==0.9.2 removed.
        venv.requirements = ''
            datasets==2.12.0
            huggingface-hub==0.14.1
            InstructorEmbedding==1.0.0
            lightgbm==3.3.5
            orjson==3.9.0
            pandas==2.0.1
            rich==13.3.5
            rouge==1.0.1
            sacrebleu==2.3.1
            scikit-learn==1.2.2
            sentencepiece==0.1.99
            sentence-transformers==2.2.2
            torch==2.0.1
            torchaudio==2.0.2
            torchvision==0.15.2
            tqdm==4.65.0
            transformers==4.29.2
            xgboost==1.7.5
            langchain==0.0.180
        '';
        };

    languages.rust = {
        enable = true;
        version = "stable";
    };

    # https://devenv.sh/pre-commit-hooks/
    # pre-commit.hooks.shellcheck.enable = true;

    # https://devenv.sh/processes/
    # processes.ping.exec = "ping example.com";

    # See full reference at https://devenv.sh/reference/options/
}
```

- Create or replace a file named `devenv.yaml` containing

```yaml
inputs:
  nixpkgs:
    url: github:NixOS/nixpkgs/nixpkgs-unstable
  nixpkgs-python:
    url: github:cachix/nixpkgs-python
  fenix:
    url: github:nix-community/fenix
    inputs:
      nixpkgs:
        follows: nixpkgs
```

### Step 3: Work within the environment

- Start `devenv` from the terminal (assuming `sh` compatible shell): `NIXPKGS_ALLOW_UNFREE=1 CARGO_HOME=$(pwd)/.cargo_home devenv shell`.
  Exiting the enviroment is the same as exiting any shell (e.g. `ctrl-D`). Note `NIXPKGS_ALLOW_UNFREE=1` is required for the installation
  of the NVidia compiler `nvcc`.

  This will take a few minutes the first time, after which everything will be cached for faster start.

- Make sure that the local Cargo binaries are available: `export PATH=${DEVENV_ROOT}/.cargo_home/bin:${PATH}`

- Compile the `pgml` extension:

```shell
cd pgml-extension

# Install pgrx locally
nice -n 19 cargo install cargo-pgrx --version "0.9.2" --locked

# devenv.nix provides PSQL version 15.
nice -n 19 cargo pgrx init --pg15=$(which pg_config)
nice -n 19 cargo pgrx install --pg-config $(which pg_config)

```
