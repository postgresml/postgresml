---
description: >-
  Today we’re taking the next steps towards open source AI becoming the industry standard. We’re releasing Llama 3.1 405B, the first frontier-level open source AI model, as well as new and improved Llama 3.1 70B and 8B models. In addition to having significantly better cost/performance relative to closed models.
featured: false
tags: [engineering]
image: ".gitbook/assets/image (2) (2).png"
---

# Announcing Support for Meta Llama 3.1

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Montana Low

July 23, 2024

We're pleased to offer Meta Llama 3.1 running in our serverless cloud today. Mark Zuckerberg explained [his company's reasons for championing open source AI](https://about.fb.com/news/2024/07/open-source-ai-is-the-path-forward/), and it's great to see a strong ecosystem forming. These models are now available in our serverless cloud with optimized kernels for maximum throughput. 

- meta-llama/Meta-Llama-3.1-8B-Instruct
- meta-llama/Meta-Llama-3.1-70B-Instruct
- meta-llama/Meta-Llama-3.1-405B-Instruct

## Is open-source AI right for you?

We think so. Open-source models have made remarkable strides, not only catching up to proprietary counterparts but also surpassing them across multiple domains. The advantages are clear:

* **Performance & reliability:** Open-source models are increasingly comparable or superior across a wide range of tasks and performance metrics. Mistral and Llama-based models, for example, are easily faster than GPT 4. Reliability is another concern you may reconsider leaving in the hands of OpenAI. OpenAI’s API has suffered from several recent outages, and their rate limits can interrupt your app if there is a surge in usage. Open-source models enable greater control over your model’s latency, scalability and availability. Ultimately, the outcome of greater control is that your organization can produce a more dependable integration and a highly reliable production application.
* **Safety & privacy:** Open-source models are the clear winner when it comes to security sensitive AI applications. There are [enormous risks](https://www.infosecurity-magazine.com/news-features/chatgpts-datascraping-scrutiny/) associated with transmitting private data to external entities such as OpenAI. By contrast, open-source models retain sensitive information within an organization's own cloud environments. The data never has to leave your premises, so the risk is bypassed altogether – it’s enterprise security by default. At PostgresML, we offer such private hosting of LLM’s in your own cloud.
* **Model censorship:** A growing number of experts inside and outside of leading AI companies argue that model restrictions have gone too far. The Atlantic recently published an [article on AI’s “Spicy-Mayo Problem'' ](https://www.theatlantic.com/ideas/archive/2023/11/ai-safety-regulations-uncensored-models/676076/) which delves into the issues surrounding AI censorship. The titular example describes a chatbot refusing to return commands asking for a “dangerously spicy” mayo recipe. Censorship can affect baseline performance, and in the case of apps for creative work such as Sudowrite, unrestricted open-source models can actually be a key differentiating value for users.
* **Flexibility & customization:** Closed-source models like GPT3.5 Turbo are fine for generalized tasks, but leave little room for customization. Fine-tuning is highly restricted. Additionally, the headwinds at OpenAI have exposed the [dangerous reality of AI vendor lock-in](https://techcrunch.com/2023/11/21/openai-dangers-vendor-lock-in/). Open-source models such as MPT-7B, Llama V2 and Mistral 7B are designed with extensive flexibility for fine tuning, so organizations can create custom specifications and optimize model performance for their unique needs. This level of customization and flexibility opens the door for advanced techniques like DPO, PPO LoRa and more.

For a full list of models available in our cloud, check out our [plans and pricing](/pricing).

