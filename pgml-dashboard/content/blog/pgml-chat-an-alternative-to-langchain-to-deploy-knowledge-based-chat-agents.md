---
author: Santi Adavani
description: 
image: https://postgresml.org/dashboard/static/images/blog/llm_based_pipeline_hero.png
image_alt: "pgml-chat: A database-driven command-line tool for deploying knowledge-based chatbots"
---
# pgml-chat: A database-driven command-line tool for deploying knowledge-based chatbots
<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/santi.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
    <p class="m-0">Santi Adavani</p>
    <p class="m-0">July 13, 2023</p>
  </div>
</div>

# Introduction
Language models like GPT-3 seem really intelligent at first, but they have a huge blindspot - no external knowledge or memory. Ask them about current events or niche topics and they just can't keep up. To be truly useful in real applications, these large language models (LLMs) need knowledge added to them somehow. The trick is getting them that knowledge fast enough to have natural conversations. Open source tools like LangChain try to help by giving language models more context and knowledge. But they end up glueing together different services into a complex patchwork. This leads to a lot of infrastructure overhead, maintenance needs, and slow response times that hurt chatbot performance. We need a better solution tailored specifically for chatbots to inject knowledge in a way that's fast, relevant and integrated.

# The 3 Pillars of a Knowledgeable Chatbot
Building a knowledgeable chatbot is like constructing a skyscraper – it requires a strong foundation. Current open source tools treat each piece of infrastructure like a Lego block for you to glue together. But this creates a precarious tower of complexity.

To truly support an intelligent conversationalist, three pillars are needed:

- A flexible knowledge base to store, organize and connect all content – documents, passages and metadata like topics, authors, urls etc.
- Embedded intelligence to understand concepts in text and index for fast discovery.
- Rapid retrieval to find just what the chatbot needs to know in the moment.

Rather than duct-taping disjointed services, what if your knowledge base was engineered from the ground up for machine learning?

PostgresML database-first approach provides:

- A home for documents and metadata in their native formats.
- Built-in functionality to chunk text and generate semantic embeddings on demand.
- Indexing and querying focused on the chatbot's needs for speed and relevance.

With the foundation consolidated into one GPU-accelerated database, you reduce complexity and maintenance dramatically while optimizing for the chatbot's success.