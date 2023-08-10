---
author: Santi Adavani
description: 
image: https://postgresml.org/dashboard/static/images/blog/llm_based_pipeline_hero.png
image_alt: "pgml-chat: A command-line tool for deploying responsive knowledge-based chatbots"
---
# pgml-chat: A command-line tool for deploying responsive knowledge-based chatbots
<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/santi.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
    <p class="m-0">Santi Adavani</p>
    <p class="m-0">July 13, 2023</p>
  </div>
</div>

# Introduction
Language models like GPT-3 seem really intelligent at first, but they have a huge blindspot - no external knowledge or memory. Ask them about current events or niche topics and they just can't keep up. To be truly useful in real applications, these large language models (LLMs) need knowledge added to them somehow. The trick is getting them that knowledge fast enough to have natural conversations. Open source tools like LangChain try to help by giving language models more context and knowledge. But they end up glueing together different services into a complex patchwork. This leads to a lot of infrastructure overhead, maintenance needs, and slow response times that hurt chatbot performance. We need a better solution tailored specifically for chatbots to inject knowledge in a way that's fast, relevant and integrated.

# Steps to build a chatbot on your own data

## 1. Building the Knowledge Base

This offline setup lays the foundation for your chatbot's intelligence. It involves:

- Gathering domain documents like articles, reports, and websites to teach your chatbot about the topics it will encounter.
- Splitting these documents into smaller chunks using segmentation algorithms. This keeps each chunk within the context size limits of AI models.
- Generating semantic embeddings for each chunk using deep learning models like SentenceTransformers. The embeddings capture conceptual meaning.
- Indexing the chunk embeddings for efficient similarity search during conversations.

This knowledge base setup powers the contextual understanding for your chatbot. It's compute-intensive but only needs to be done once.

## 2. Connecting to Conversational AI

With its knowledge base in place, now the chatbot links to models that allow natural conversations:

- Querying the indexed chunks to rapidly pull the most relevant passages for answering users' questions.
- Passing those passages to a model like GPT-3 to generate conversational responses.
- Orchestrating the query, retrieval and generation flow to enable real-time chat.
