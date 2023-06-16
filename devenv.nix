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
