---
author: Montana Low
description: GPTQ & GGML allow PostgresML to fit larger models in less RAM, and perform inference significantly faster on NVIDIA, Apple and Intel hardware.
image: https://postgresml.org/dashboard/static/images/blog/discrete_quantization.jpg
image_alt: We read to learn
---

# Announcing GPTQ & GGML Quantized LLM support for Huggingface Transformers 

<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/montana.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
    <p class="m-0">Montana Low</p>
    <p class="m-0">June 20, 2023</p>
  </div>
</div>

## Introduction

Large Language Models (LLMs) are... large. They have a lot of parameters, which make up the weights and biases of the layers inside their deep neural networks. Typically, these parameters are represented individual 32-bit floating point numbers, so a model like GPT-2 that has 1.5B parameters would need `4 bytes * 1,500,000,000 = 6GB RAM`. The Leading Open Source models like LLaMA, Alpaca, and Guanaco, currently have 65B parameters, which would require about 260GB RAM. This is a lot of RAM, and it's not even counting the RAM needed to store the input and output data if you're doing a large processing job.

Bandwidth between RAM and CPU often becomes a bottleneck for performing inference with these models, rather than the number of processing cores or their speed, because the processors become starved for data. One way to reduce the amount of RAM needed is to use a smaller datatype, like 16-bit floating point numbers, which would reduce the RAM needed by half. There are a couple competing 16-bit standards, but NVIDIA has introduced support for bfloat16 in their latest hardware generation, which keeps the full exponential range of float32, but gives up a 2/3rs of the precision. Most research has shown this is a good quality/performance tradeoff, and that model outputs are not terribly sensitive when truncating the least significant bits.

| Format   | Significand | Exponent |
|----------|-------------|----------|
| bfloat16 | 8 bits      | 8 bits   |
| float16  | 11 bits     | 5 bits   |
| float32  | 24 bits     | 8 bits   |

![discrete_quantization.jpg](/dashboard/static/images/blog/discrete_quantization.webp)
<center><i>Discrete quantization is not a new idea. It's been used by both algorithms and artists for hundreds of years.</i></center>

## Quantization

Going beyond 16 bit down to 8 or 4 bits is possible, but requires more work. This is the process of _quantization_. Quantization works by first training the model with 32-bit floats, then converting the weights to smaller integer primitives that will still benefit from hardware accelerated instruction sets like Intel's [AVX](https://en.wikipedia.org/wiki/Advanced_Vector_Extensions). A simple way to quantize a model can be done by first finding the maximum and minimum values of the weights, then dividing the range of values into the number of buckets available in your integer type, 256 for 8-bit, 16 for 4-bit. This is called _post-training quantization_, and it's the simplest way to quantize a model. 

[GPTQ: Accurate Post-Training Quantization for Generative Pre-trained Transformers](https://arxiv.org/abs/2210.17323) is a research paper that outlines the details for quantizing LLMs after they have already been trained on full float32 precision, and the tradeoffs involved. Their work is implemented as an [open source library](https://github.com/IST-DASLab/gptq), which has been adapted to work with Huggingface Transformers by [AutoGPTQ](https://github.com/PanQiWei/AutoGPTQ). PostgresML will automatically use AutoGPTQ when a HuggingFace model with GPTQ in the name is used.

[GGML](https://github.com/ggerganov/ggml) is another quantization implementation focused more on CPU optimization, particularly for Apple M1 & M2 silicon. It relies on the same principles but is a different underlying implementation. As a general rule of thumb, if you're using NVIDIA hardware and your entire model will fit in VRAM, GPTQ will be faster. If you're using Apple or Intel hardware, GGML will likely be faster.

The community (particular shoutout to [TheBloke](https://huggingface.co/TheBloke)), has been applying these quantization methods to the LLMs in the Huggingface Transformers library, so many versions of your favorite LLMs are now available in more efficient formats. This might allow you to move up to a larger model size, or fit more models in the same amount of RAM.

## Using GPTQ & GGML in PostgresML

You'll need to update to PostgresML 2.6.0 or later to use GPTQ or GGML. AutoGPTQ also provides prebuilt wheels for Python if you're having trouble installing the pip package which builds it from source. They maintain a list of wheels [available for download](https://github.com/PanQiWei/AutoGPTQ/releases) on GitHub. You can update your Python dependencies for PostgresML to take advantage of these new capabilities:

```commandline
pip install -r requirements.txt
```

PostgresML will automatically use GPTQ or GGML when a HuggingFace model has one of the libraries in its name.

### GPTQ - GPU

!!! generic

!!! code_block

```sql
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

!!! results "281.213 ms"
                                                                               transform
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------
 ["Once upon a time, the world was a place of great beauty and great danger. The world was a place of great danger. The world was a place of great danger. The world"]
!!!

!!!

### GGML - GPU

!!! generic

!!! code_block

```sql
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

!!! results "252.213 ms"
                                                                        transform
----------------------------------------------------------------------------------------------------------------------------------------------------------
 [" the world was filled with people who were not only rich but also powerful.\n\nThe first thing that came to mind when I thought of this place is how"]
!!!

!!!

### Vanilla GPT-2 - GPU

!!! generic

!!! code_block

```sql
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

!!! results "279.888 ms"
                                                                                                                                                                                   transform

---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------------------
 [[{"Once upon a time, I'd get angry over the fact that my house was going to have some very dangerous things from outside. To be honest, I know it's going to be"}]]
!!!

!!!

This quick example running on my RTX 3090 GPU shows there is very little difference in runtime for these libraries and models when everything fits in VRAM by default. But let's see what happens when we execute the model on my Intel i9-13900 instead...

### GGML - CPU

!!! generic

!!! code_block

```sql
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

!!! results "266.997 ms"
                                                                                 transform
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 [" we've all had an affair with someone and now the truth has been revealed about them. This is where our future comes in... We must get together as family"]
!!!

### Vanilla GPT-2 - CPU

!!! generic

!!! code_block

```sql
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

!!! results "33224.136 ms"
                                                                                                  transform
--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 [[{"generated_text": "Once upon a time, we were able, due to our experience at home, to put forward the thesis that we're essentially living life as a laboratory creature with the help of other humans"}]]
!!!

!!!

Now you can see the difference. With both models forced to use only the CPU, we can see that a quantized version can be literally 100x faster. In fact, the quantized version on the CPU is as fast as the vanilla version on the GPU. This is a huge win for CPU users.

### Conclusion

There are a ton of great open source LLMs. If you're looking for a list to try, check out [the leaderboard](https://huggingface.co/spaces/HuggingFaceH4/open_llm_leaderboard). You can also [search for GPTQ](https://huggingface.co/models?search=gptq) and [GGML](https://huggingface.co/models?search=ggml) versions of those models on the hub to see what is popular in the community. If you're looking for a model that is not available in a quantized format, you can always quantize it yourself. If you're successful, please consider sharing your quantized model with the community! 


