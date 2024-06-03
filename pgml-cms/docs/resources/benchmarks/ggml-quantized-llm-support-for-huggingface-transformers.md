---
description: Quantization allows PostgresML to fit larger models in less RAM.
---

# GGML Quantized LLM support for Huggingface Transformers

Quantization allows PostgresML to fit larger models in less RAM. These algorithms perform inference significantly faster on NVIDIA, Apple and Intel hardware. Half-precision floating point and quantized optimizations are now available for your favorite LLMs downloaded from Huggingface.

## Introduction

Large Language Models (LLMs) are... large. They have a lot of parameters, which make up the weights and biases of the layers inside deep neural networks. Typically, these parameters are represented by individual 32-bit floating point numbers, so a model like GPT-2 that has 1.5B parameters would need `4 bytes * 1,500,000,000 = 6GB RAM`. The Leading Open Source models like LLaMA, Alpaca, and Guanaco, currently have 65B parameters, which requires about 260GB RAM. This is a lot of RAM, and it's not even counting what's needed to store the input and output data.

Bandwidth between RAM and CPU often becomes a bottleneck for performing inference with these models, rather than the number of processing cores or their speed, because the processors become starved for data. One way to reduce the amount of RAM and memory bandwidth needed is to use a smaller datatype, like 16-bit floating point numbers, which would reduce the model size in RAM by half. There are a couple competing 16-bit standards, but NVIDIA has introduced support for bfloat16 in their latest hardware generation, which keeps the full exponential range of float32, but gives up a 2/3rs of the precision. Most research has shown this is a good quality/performance tradeoff, and that model outputs are not terribly sensitive when truncating the least significant bits.

| Format      | Significand | Exponent |
| ----------- | ----------- | -------- |
| bfloat16    | 8 bits      | 8 bits   |
| float16     | 11 bits     | 5 bits   |
| float32     | 24 bits     | 8 bits   |
| <p><br></p> |             |          |

You can select the data type for torch tensors in PostgresML by setting the `torch_dtype` parameter in the `pgml.transform` function. The default is `float32`, but you can also use `float16` or `bfloat16`. Here's an example of using `bfloat16` with the [Falcon-7B Instruct](https://huggingface.co/tiiuae/falcon-7b-instruct) model:

!!! generic

!!! code\_block time="4584.906 ms"

```postgresql
SELECT pgml.transform(
    task => '{
        "model": "tiiuae/falcon-7b-instruct",
        "device_map": "auto",
        "torch_dtype": "bfloat16",
        "trust_remote_code": true
     }'::JSONB,
     args => '{
        "max_new_tokens": 100
     }'::JSONB,
     inputs => ARRAY[
        'Complete the story: Once upon a time,'
     ]
) AS result;
```

!!!

!!! results

| transform                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[\[{"generated\_text": "Complete the story: Once upon a time, there was a small village where everyone was happy and lived peacefully.\nOne day, a powerful ruler from a neighboring kingdom arrived with an evil intent. He wanted to conquer the peaceful village and its inhabitants. The ruler was accompanied by a large army, ready to take control. The villagers, however, were not to be intimidated. They rallied together and, with the help of a few brave warriors, managed to defeat the enemy. The villagers celebrated their victory, and peace was restored in the kingdom for"}]] |

!!!

!!!

4.5 seconds is slow for an interactive response. If we're building dynamic user experiences, it's worth digging deeper into optimizations.

## Quantization

_Discrete quantization is not a new idea. It's been used by both algorithms and artists for more than a hundred years._\\

Going beyond 16-bit down to 8 or 4 bits is possible, but not with hardware accelerated floating point operations. If we want hardware acceleration for smaller types, we'll need to use small integers w/ vectorized instruction sets. This is the process of _quantization_. Quantization can be applied to existing models trained with 32-bit floats, by converting the weights to smaller integer primitives that will still benefit from hardware accelerated instruction sets like Intel's [AVX](https://en.wikipedia.org/wiki/Advanced\_Vector\_Extensions). A simple way to quantize a model can be done by first finding the maximum and minimum values of the weights, then dividing the range of values into the number of buckets available in your integer type, 256 for 8-bit, 16 for 4-bit. This is called _post-training quantization_, and it's the simplest way to quantize a model.

[GPTQ: Accurate Post-Training Quantization for Generative Pre-trained Transformers](https://arxiv.org/abs/2210.17323) is a research paper that outlines the details for quantizing LLMs after they have already been trained on full float32 precision, and the tradeoffs involved. Their work is implemented as an [open source library](https://github.com/IST-DASLab/gptq), which has been adapted to work with Huggingface Transformers by [AutoGPTQ](https://github.com/PanQiWei/AutoGPTQ). PostgresML will automatically use AutoGPTQ when a HuggingFace model with GPTQ in the name is used.

[GGML](https://github.com/ggerganov/ggml) is another quantization implementation focused on CPU optimization, particularly for Apple M1 & M2 silicon. It relies on the same principles, but is a different underlying implementation. As a general rule of thumb, if you're using NVIDIA hardware and your entire model will fit in VRAM, GPTQ will be faster. If you're using Apple or Intel hardware, GGML will likely be faster.

The community (shoutout to [TheBloke](https://huggingface.co/TheBloke)), has been applying these quantization methods to LLMs in the Huggingface Transformers library. Many versions of your favorite LLMs are now available in more efficient formats. This might allow you to move up to a larger model size, or fit more models in the same amount of RAM.

## Using GPTQ & GGML in PostgresML

You'll need to update to PostgresML 2.6.0 or later to use GPTQ or GGML. You will need to update your Python dependencies for PostgresML to take advantage of these new capabilities. AutoGPTQ also provides prebuilt wheels for Python if you're having trouble installing the pip package which builds it from source. They maintain a list of wheels [available for download](https://github.com/PanQiWei/AutoGPTQ/releases) on GitHub.

```commandline
pip install -r requirements.txt
```

### GPU Support

PostgresML will automatically use GPTQ or GGML when a HuggingFace model has one of those libraries in its name. By default, PostgresML uses a CUDA device where possible.

#### GPTQ

!!! generic

!!! code\_block time="281.213 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "mlabonne/gpt2-GPTQ-4bit"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                                                              |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \["Once upon a time, the world was a place of great beauty and great danger. The world was a place of great danger. The world was a place of great danger. The world"] |

!!!

!!!

#### GGML

!!! generic

!!! code\_block time="252.213 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "marella/gpt-2-ggml"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                                                 |
| --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[" the world was filled with people who were not only rich but also powerful.\n\nThe first thing that came to mind when I thought of this place is how"] |

!!!

!!!

#### GPT2

!!! generic

!!! code\_block time="279.888 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "gpt2"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                                                              |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[\[{"Once upon a time, I'd get angry over the fact that my house was going to have some very dangerous things from outside. To be honest, I know it's going to be"}]] |

!!!

!!!

This quick example running on my RTX 3090 GPU shows there is very little difference in runtime for these libraries and models when everything fits in VRAM by default. But let's see what happens when we execute the model on my Intel i9-13900 CPU instead of my GPU...

### CPU Support

We can specify the CPU by passing a `"device": "cpu"` argument to the `task`.

#### GGML

!!! generic

!!! code\_block time="266.997 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "marella/gpt-2-ggml",
      "device": "cpu"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                                                      |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[" we've all had an affair with someone and now the truth has been revealed about them. This is where our future comes in... We must get together as family"] |

!!!

!!!

#### GPT2

!!! generic

!!! code\_block time="33224.136 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "gpt2",
      "device": "cpu"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                                                                                                       |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[\[{"generated\_text": "Once upon a time, we were able, due to our experience at home, to put forward the thesis that we're essentially living life as a laboratory creature with the help of other humans"}]] |

!!!

!!!

Now you can see the difference. With both implementations and models forced to use only the CPU, we can see that a quantized version can be literally 100x faster. In fact, the quantized version on the CPU is as fast as the vanilla version on the GPU. This is a huge win for CPU users.

### Larger Models

HuggingFace and these libraries have a lot of great models. Not all of these models provide a complete config.json, so you may need to include some additional params for the task, like `model_type`.

#### LLaMA

!!! generic

!!! code\_block time="3411.324 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "TheBloke/robin-7B-v2-GGML",
      "model_type": "llama"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                              |
| -------------------------------------------------------------------------------------------------------------------------------------- |
| \[" in a land far away, there was a kingdom ruled by a wise and just king. The king had three sons, each of whom he loved dearly and"] |

!!!

!!!

#### MPT

!!! generic

!!! code\_block time="4198.817 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "TheBloke/MPT-7B-Storywriter-GGML",
      "model_type": "mpt"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                |
| ------------------------------------------------------------------------------------------------------------------------ |
| \["\n\nWhen he heard a song that sounded like this:\n\n"The wind is blowing, the rain's falling. \nOh where'd the love"] |

!!!

!!!

#### Falcon

!!! generic

!!! code\_block time="4198.817 ms"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "TheBloke/falcon-40b-instruct-GPTQ",
      "trust_remote_code": true
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                |
| ------------------------------------------------------------------------------------------------------------------------ |
| \["\n\nWhen he heard a song that sounded like this:\n\n"The wind is blowing, the rain's falling. \nOh where'd the love"] |

!!!

!!!

### Specific Quantization Files

Many of these models are published with multiple different quantization methods applied and saved into different files in the same model space, e.g. 4-bit, 5-bit, 8-bit. You can specify which quantization method you want to use by passing a `model_file` argument to the `task`, in addition to the `model`. You'll need to check the model card for file and quantization details.

!!! generic

!!! code\_block time="6498.597"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "TheBloke/MPT-7B-Storywriter-GGML",
      "model_file": "mpt-7b-storywriter.ggmlv3.q8_0.bin"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                                          |
| -------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[" we made peace with the Romans, but they were too busy making war on each other to notice. The king and queen of Rome had a son named Romulus"] |

!!!

!!!

### The whole shebang

PostgresML aims to provide a flexible API to the underlying libraries. This means that you should be able to pass in any valid arguments to [`AutoModel.from_pretrained(...)`](https://huggingface.co/docs/transformers/v4.30.0/en/model\_doc/auto#transformers.FlaxAutoModelForVision2Seq.from\_pretrained) via the `task`, and additional arguments to call on the resulting pipeline during inference for `args`. PostgresML caches each model based on the `task` arguments, so calls to an identical task will be as fast as possible. The arguments that are valid for any model depend on the inference implementation it uses. You'll need to check the model card and underlying library for details.

Getting GPU acceleration to work may also depend on compiling dependencies or downloading Python wheels as well as passing in the correct arguments if your implementing library does not run on a GPU by default like huggingface transformers. PostgresML will cache your model on the GPU, and it will be visible in the process list if it is being used, for as long as your database connection is open. You can always check `nvidia-smi` to see if the GPU is being used as expected. We understand this isn't ideal, but we believe the bleeding edge should be accessible to those that dare. We test many models and configurations to make sure our cloud offering has broad support, but always appreciate GitHub issues when something is missing.

Shoutout to [Tostino](https://github.com/Tostino/) for the extended example below.

!!! generic

!!! code\_block time="3784.565"

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "TheBloke/vicuna-7B-v1.3-GGML",
      "model_type": "llama",
      "model_file": "vicuna-7b-v1.3.ggmlv3.q5_K_M.bin",
      "gpu_layers": 256
    }'::JSONB,
    inputs => ARRAY[
        $$A chat between a curious user and an artificial intelligence assistant. The assistant gives helpful, detailed, and polite answers to the user's questions.

USER: Please write an intro to a story about a woman living in New York.
ASSISTANT:$$
    ],
    args => '{
      "max_new_tokens": 512,
          "threads": 16,
      "stop": ["USER:","USER"]
    }'::JSONB
);
```

!!!

!!! results

| transform                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| \[" Meet Sarah, a strong-willed woman who has always had a passion for adventure. Born and raised in the bustling city of New York, she was no stranger to the hustle and bustle of life in the big apple. However, Sarah longed for something more than the monotonous routine that had become her daily life.\n\nOne day, while browsing through a travel magazine, Sarah stumbled upon an ad for a wildlife conservation program in Africa. Intrigued by the opportunity to make a difference in the world and expand her horizons, she decided to take the leap and apply for the position.\n\nTo her surprise, Sarah was accepted into the program and found herself on a plane bound for the African continent. She spent the next several months living and working among some of the most incredible wildlife she had ever seen. It was during this time that Sarah discovered a love for exploration and a desire to see more of the world.\n\nAfter completing her program, Sarah returned to New York with a newfound sense of purpose and ambition. She was determined to use her experiences to fuel her love for adventure and make the most out of every opportunity that came her way. Whether it was traveling to new destinations or taking on new challenges in her daily life, Sarah was not afraid to step outside of her comfort zone and embrace the unknown.\n\nAnd so, Sarah's journey continued as she made New York her home base for all of her future adventures. She became a role model for others who longed for something more out of life, inspiring them to chase their dreams and embrace the exciting possibilities that lay ahead."] |

!!!

!!!

### Conclusion

There are many open source LLMs. If you're looking for a list to try, check out [the leaderboard](https://huggingface.co/spaces/HuggingFaceH4/open\_llm\_leaderboard). You can also [search for GPTQ](https://huggingface.co/models?search=gptq) and [GGML](https://huggingface.co/models?search=ggml) versions of those models on the hub to see what is popular in the community. If you're looking for a model that is not available in a quantized format, you can always quantize it yourself. If you're successful, please consider sharing your quantized model with the community!

To dive deeper, you may also want to consult the docs for [ctransformers](https://github.com/marella/ctransformers) if you're using a GGML model, and [auto\_gptq](https://github.com/PanQiWei/AutoGPTQ) for GPTQ models. While Python dependencies are fantastic to let us all iterate quickly, and rapidly adopt the latest innovations, they are not as performant or resilient as native code. There is good progress being made to move a lot of this functionality into [rustformers](https://github.com/rustformers/llm) which we intend to adopt on our quest to remove Python completely on the road to PostgresML 3.0, but we're not going to slow down the pace of innovation while we build more stable and performant APIs.

GPTQ & GGML are a huge win for performance and memory usage, and we're excited to see what you can do with them.
