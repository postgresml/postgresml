<style>
    label img {
        position: relative;
        top: 0.3em;
        left: -0.1em;
        height: auto !important;
        width: 1.2em !important;
    }
</style>

# 🤗 Transformers
PostgresML integrates Hugging Face to bring state-of-the-art models into the data layer. Hugging Face provides a library with tens of thousands of pre-trained models, and Pipelines to turn raw inputs into useful results. Many state of the art architectures have been published and made available for download. You will want to browse all the architectures available on [🤗 Hugging Face](https://huggingface.co/tasks) to find the perfect solution for your dataset and task.

## Setup
Install the machine learning depedencies on the database for the transformers you would like to use:

=== "![Pytorch Logo](/images/logos/pytorch.svg) PyTorch"

    See the [Pytorch docs](https://pytorch.org/) for more information.

    ```bash
    $ sudo pip3 install torch
    ```

=== "![Tensorflow Logo](/images/logos/tensorflow.svg) Tensorflow"

    See the [Tensorflow docs](https://www.tensorflow.org/install/) for more information

    ```bash
    $ sudo pip3 install tensorflow
    ```

=== "![Flax](/images/logos/flax.png) Flax"

    See the [Flax docs](https://flax.readthedocs.io/en/latest/installation.html) for more information

    ```bash
    $ sudo pip3 install flax
    ```

Models will be downloaded and cached on the database for repeated usage. View the [Tranmsformers installation docs](https://huggingface.co/docs/transformers/installation) for cache management details and offline deployments.

You may also want to [install GPU support](/user_guides/setup/gpu_support.md) when working with larger models.

## Standard Datasets
Many datasets have been published to stimulate research and benchmark architectures, but also help demonstrate API usage in the tutorials. The Datasets package provides a way to load published datasets into Postgres:

```bash
$ sudo pip install datasets
```

## Audio Processing
Torch Audio is required for many models that process audio data. You can install the additional dependencies:

```bash
$ sudo pip install torchaudio
```

