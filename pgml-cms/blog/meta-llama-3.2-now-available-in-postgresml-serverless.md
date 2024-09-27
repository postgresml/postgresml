---
description: Bringing smaller, smarter models to your data.
featured: true
tags: [product]
image: ".gitbook/assets/Blog-Image_Llama-3.2.jpg"
---

# Llama 3.2 now available in PostgresML serverless

<div align="left">

<figure><img src=".gitbook/assets/image.png" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Cassandra Stummer

September 27, 2024
  
Today, we're excited to announce that PostgresML now supports Llama 3.2, a development that not only enhances our capabilities, but also aligns with our core philosophy: bring the models to your data, not the other way around.

## The power of smaller models

The AI market is finally moving away from the "bigger is better" mentality. Size no longer equals capability. While companies like OpenAI pushed the research frontier with massive models, we're now seeing open-source models 225 times smaller achieving capabilities comparable to GPT-4 at launch. This shift challenges the notion that enormous, closed source models are the only path to advanced AI.

## Why Llama 3.2 in PostgresML?

Companies aiming to run their own models face a critical challenge. Data sources for interactive AI are hard to scale. The amount of context models need is growing: text, vectors, images, user history; find the needles in multiple haystacks, on demand. Gathering and sorting through context from growing data sources becomes the bottleneck in the system. 

As models become smaller and datasets grow larger, the traditional approach of moving data to models becomes increasingly inefficient. That’s why we've always believed that the future of AI lies in bringing models directly to your data. The integration of smaller models like Llama 3.2  into PostgresML is a testament to our vision of the future of AI: Big data and small models colocating to deliver the most efficient, scalable AI infrastructure. 

## What this means for you

The Instruct variants, LLama 3.2 1B and 3B, are now standard models included with all Serverless Databases at **no additional cost**. You can try them now. 

## Getting Started

Integrating Llama 3.2 with PostgresML is straightforward. Here's a quick example:

```postgresql
SELECT pgml.transform(
  task   => '{
    "task": "text-generation",
    "model": "meta-llama/Llama-3.2-3B-Instruct"
  }'::JSONB,
  inputs  => Array['AI is going to'] 
);
```

## The road ahead

This is just the beginning. We're committed to continually supporting the latest and greatest models, always with the goal of making AI more efficient, and aligned with your data strategy.

Ready to experience the power of Llama 3.2 in PostgresML? Get started today or contact our team for a personalized demo.

Stay tuned for more updates as we continue to push the boundaries of what's possible with AI in databases\!  
