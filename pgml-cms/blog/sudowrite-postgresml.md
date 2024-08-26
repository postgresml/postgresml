---
description: How the best AI-powered app for fiction writers built their winning RAG stack
featured: true
tags: []
image: ".gitbook/assets/sudowrite-pgml_blog-image.png"
---

# Sudowrite + PostgresML

<div align="left">

<figure><img src=".gitbook/assets/image.png" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Cassandra Stummer

August 26, 2024

## The challenge

[Sudowrite](https://www.sudowrite.com/) is an AI-powered writing assistant that helps author's craft compelling stories and overcome writer's block. They wanted to give authors a cool new feature: the ability to chat with an AI editor about their stories. 

James Yu, Sudowrite’s founder and CTO, knew that meant standing up a RAG (retrieval augmented generation) system. RAG is a cutting-edge AI technique, but James was searching for a solution that worked in production and at-scale, not just in the latest prototype trending on Hacker News. 

“I didn’t want to geek out about RAG for days or weeks. Just give me something that approximately works and then I can move on to the next thing.”

## Enter PostgresML

PostgresML is simple – it’s PostgreSQL with GPUs for ML/AI apps. Along with GPUs, the PostgresML Cloud provides a full-featured machine learning platform right in the database; with functionality for search, embeddings, retrieval and more. 

James was sold on the simplicity of doing AI in Postgres, the database his engineers already use and love


<div class="hide-admonition-title-container">

!!! tip

<p class="center">
    <i>"Why add yet another database to your stack if you don't have to? Being able to co-locate your data – to query across the same metadata stack – is a no brainer.”</i>
</p>

<p><i>James Yu, Founder @Sudowrite</i></p>

!!!

</div>

## Quick and easy implementation

Time to prototype was key for the Sudowrite team when testing out RAG systems. They used the Javascript SDK to get a full proof of concept chatbot fully synced to document changes in three hours flat. Once they decided to use PostgresML, it just took a few function calls with the SDK to start syncing data with production. 

“It was pretty easy,” James said. “I also just like the visibility. As it's indexing I can just refresh my Postgres and I see the chunks, I can inspect it all. It’s immediate validation.” His team knows Postgres, so there was no need to get familiar with a niche vector database service like Pinecone or Qdrant. 

James added: “I tried Pinecone and it felt very opaque - it’s a weird API and the data felt weirdly structured. I’m not going to pay exorbitant fees for a proprietary database where I’m not even sure how they’re performing the queries. I had to go through their UI, whereas for PostgresML I could visually see it in the same way as all my other data.”

And since PostgresML has ML/AI functionality built-in, they didn’t need to create complex data pipelines to connect to embedding services, data pre-processors, or other ML/AI microservices. The Sudowrite team performs embedding generation and retrieval using SQL queries, right inside their PostgresML database. 

Additionally the Sudowrite team had access to an on-call PostgresML engineer and a private slack channel with same-day responses to ensure implementation was as smooth and fast as possible. 

"The support from the PostgresML team has been top-notch," James adds. "They're always quick to respond when we have questions, and they understand our need for flexibility.” 

## The results: In-database AI is a win for devs and users

With PostgresML in place, Sudowrite's new AI chatbot feature is already making waves:

- Sudowrite's RAG system makes more than 1 million calls per hour
- The engineering team is loving the streamlined operations
- A growing percentage of daily active users are chatting it up with the AI editor

Performance and scalability were initial concerns for Sudowrite, given their large document base. James recalls his pleasant surprise: **"I thought, 'wow it's really fast, it's indexing all these things.' I was skeptical at first because we had a lot of documents, but it indexed quickly and it's really performant."** 

<div class="hide-admonition-title-container">

!!! tip

<p class="center">
<i>"The quality – especially the RAG piece – has been great. In terms of scaling and everything, it’s been great."</i>
</p>

!!!

</div>

Additionally, PostgresML's integration has been seamless for Sudowrite's development team,  allowing engineers to focus on enhancing the user experience rather than wrestling with complex infrastructure. “I even have a contractor, and we handed it off to him pretty easily…And for him to be able to get up to speed was relatively painless,” James added.

This efficiency has given Sudowrite confidence in their ability to scale the chatbot feature to meet growing demand – and the Sudowrite team sees tremendous potential for further adoption: "People want more chat. We have plans to make it more up front and center in the app." 

## What's next for Sudowrite?

James and his team are just getting started. They're cooking up plans to:

- Make the chatbot even more visible in the app
- Allow authors to import their entire novel and interact with it via RAG 
- Create automated knowledge graphs from author’s stories


<div class="hide-admonition-title-container">

!!! tip

<p class="center">
<i>"PostgresML has given us a solid foundation for our product. Their RAG extends the capabilities of our LLMs. It’s an essential ingredient for us to create tools that help writers create even more amazing stories."</i>
</p>

!!!

</div>

## The bottom line

By choosing PostgresML, Sudowrite found a powerful, flexible solution that:

- Integrates seamlessly with their existing systems
- Scales effortlessly without the need for complex infra management
- Provides the transparency and flexibility to customize and expand their offering

James sums it up perfectly: "For me, PostgresML just makes a lot of sense.”
