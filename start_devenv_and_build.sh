#!/bin/env /bin/bash

# Ensures there is a local Cargo crates repo
[[ ! -d .cargo_home ]] && mkdir .cargo_home
export CARGO_HOME=$(pwd)/.cargo_home

# In case some packages are not free (e.g. Nvidia nvcc)
export NIXPKGS_ALLOW_UNFREE=1

# Create a devenv.nix with up-to-date Python requirements.txt
cat <<PROLOGUE_EOF > devenv.nix
#
# WARNING:
# DO NOT EDIT devenv.nix. THIS FILE IS AUTOMATICALLY by the startup script.
# EDIT THAT INSTEAD.
#

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
    pkgs.binutils-unwrapped
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
    # This shell will be executed _after_ the environment is activated.
    enterShell = ''
        # Post activation script
        export PATH=${DEVENV_ROOT}/.cargo_home/bin:${PATH}

        # Load and compile cargo pgrx
        cd pgml-extension
        echo ""
        echo "---- Installing cargo-pgrx..."
        nice -n 19 cargo install cargo-pgrx --version "0.9.2" --locked

        # Build extension for PSQL 15. 
        echo ""
        echo "---- Initialising cargo-pgrx..."
        nice -n 19 cargo pgrx init --pg15=$(which pg_config)

        echo ""
        echo "---- Building extension..."
        git submodule update --init --recursive
        nice -n 19 cargo build

        cd ..
    '';

    # https://devenv.sh/languages/
    languages.nix.enable = true;

    languages.python = {
        enable = true;
        version = "3.10";

        venv.enable = true;
        venv.requirements = ''

    # Add modules for Jupyter notebooks and examples. Does not modify requirements.txt
    ipykernel
    IProgress
    python-dotenv
    psycopg
    psycopg-pool
    tqdm
    
PROLOGUE_EOF


# Insert up-to-date requirements
cat pgml-extension/requirements.txt >> devenv.nix

# Insert epilogue of devenv.nix
cat <<EPILOGUE_EOF >> devenv.nix
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
EPILOGUE_EOF

# Enter devenv environment.
devenv shell

