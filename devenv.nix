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
    enterShell = ''
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
        python-dotenv
	psycopg
	psycopg-pool
	tqdm

accelerate==0.19.0
datasets==2.12.0
deepspeed==0.9.2
huggingface-hub==0.14.1
InstructorEmbedding==1.0.0
lightgbm==3.3.5
orjson==3.9.0
pandas==2.0.1
rich==13.3.5
rouge==1.0.1
sacrebleu==2.3.1
sacremoses==0.0.53
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
einops==0.6.1
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
