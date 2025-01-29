
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

- Within the top directory of the PostgresML repo, run `devenv init`.

- We will use a local home for all Cargo crates: `mkdir .cargo_home`

- The environment definition `devenv.nix` is generated dynamically by `start_devenv_and_build.sh`. It contains:

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
    pkgs.gcc-unwrapped

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

        venv.requirements = ''
            # GENERATED FROM requirements.txt
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

- Start by running: `./start_devenv_and_build.sh`.

  This will take a few minutes the first time, after which everything will be cached for faster start.

- Note: At the end of the script, you will end in the shell used in this file. Here `/bin/bash`.

```
