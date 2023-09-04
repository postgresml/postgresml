---
author: Montana Low
description: Why PostgresML is more reliable, efficient and simpler.
image: https://postgresml.org/dashboard/static/images/blog/embeddings_2.jpg
image_alt: Data is always the best medicine.
---

# Separate clients and services, not data and compute 

<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/montana.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
    <p class="m-0">Montana Low</p>
    <p class="m-0">August 4, 2023</p>
  </div>
</div>

## Introduction

Separating data from compute is a powerful scalability technique, but it is a tradeoff that sacrifices latency, reliability and simplicity. There are a couple of common examples of this tradeoff in the wild.  


If you're not careful with your architecture

you should never make more than once. It introduces additional latency, reliability, and  s Many modern databases proclaim they have separated data and compute. 
We've had compute and data separated over 

It , inefficient, and ultimately unnecessary.that can sometimes

  

We already have multiple network gaps between network attached storage and clients over the network that can be used for horizontal scalabilitygive us the same advantages.

Microservice architectures more successfully separate persistent data and computation, but when dealing with data intensive Machine Learning or Generative AI application workloads, they also become inefficient, and scaling their data source is still a requirement. Separating data from compute, instead of encapsulating data and compute together in a dedicated service, leads to what Andreessen Horowitz refers to as Unified Data Infrastructure 2.0. 

<img src="/dashboard/static/images/blog/Unified-Data-Infrastructure-2.0.webp" alt="Unified Data Infrastructure (2.0)" />
<center><p><i>Notes: Excludes OLTP, log analysis, and SaaS analytics apps.</i></p></center>

I would refer to this as the Data Industrial Complex instead, because it does not appear Unified to me. The innate domain complexity of Machine learning garners its own expanded diagram.

<img src="/dashboard/static/images/blog/Machine-Learning-Infrastructure-2.0.webp" alt="Machine Learning Infrastructure (2.0)" />
<center><p><i>Notes: Excludes OLTP, log analysis, and SaaS analytics apps.</i></p></center>

As an industry practitioner, I can confirm that a16z has done their homework. This is an accurate depiction and their analysis that the trend toward more databases, microservices and system complexity will continue, has already been proven correct. You'll notice these diagrams are missing the recently ascendant Vector Database and associated Generative AI APIs. 

Millions of engineering hours and billions of dollars are being spent shuffling data from one system to another, so that it can actually become useful. It requires many teams to manage these systems, and those teams require their own management. The value of data is so important to the bottom line, that it's worth the cost of all this complexity. But it doesn't have to be this way.

## The secret to PostgresML's success

[Figma scaled](https://www.figma.com/blog/how-figma-scaled-to-multiple-databases/) to $2B in revenue, with a single 48 core Postgres database, before taking on more sophisticated scaling complexity. AWS offers single instances with 192 cores these days, which implies you might consider sticking with a single Postgres workhorse until you're doing something like $8B in revenue, at which point, you too will be able to afford "experts" to pay off any technical debt incurred along the way.

The more efficient way to achieve scalability and reliability is via separating clients from services, rather than data from compute. Clients are naturally separated from databases over the network, so you can put your scalability logic in that existing gap, rather than creating a new one inside the database that will introduce latency, as well as network and logical errors.

Postgres supports several forms of replication that can be tuned for both transactional and analytical workloads. Our Postgres Pooler PgCat encapsulates the complexity of replication, failover, sharding, and other distributed systems concerns from clients. This allows Postgres to scale horizontally, while maintaining a safe, simple and efficient core.  

Put data and compute together in a dedicated service, and then replicate the data along with compute to scale horizontally. 

This is the case for PgCat.

The corollary, is that when data becomes large, it's better to move the logic to the data than move the data to the logic. This is the case for Machine Learning systems, which are relatively data intensive. Chatty ML microservices often starve their relative expensive hardware, while waiting on data over the network. This is the reason we created PostgresML.

When you combine PgCat and PostgresML on top of the fastest growing open source database ecosystem, you get a system that is more reliable, more efficient, and simpler than anything comparable in terms of capability. It's scalable, not just in terms of hardware, or financial costs, but in terms of the human resources required for administration.









PostgresML wins real world benchmarks by orders of magnitude, with a fraction of the resources, because we separate clients and services, instead of data from compute. _The secret to our success is PgCat_. Unless you're running a database that is doing 100,000 transactions per second, you probably don't even need PgCat, although you may need some tuning. You can just get a bigger machine, and Postgres will be simpler, more reliable and more efficient than any other database on the market for **all** your workloads.
 
<it's an older code sir, but it checks out>

### PgCat




There is a popular notion that if we separate data and compute, we can containerize all the things, and then horizontally scale without any other concerns. This usually means containerizing the compute, and then shunting the data off _somewhere else_ along with the actual hard problems involved in scalable, consistent, persistent data, while loudly pronouncing VICTORY!





PostgresML is relatively fast, scalable, reliable. It's also relatively simple. How does it win uncompromisingly?


Data Scientists are often surprisingly insightful and intelligent. They're also often surprisingly terrible at software and data engineering. This is a problem because our industry often looks to Data Scientists, sometimes rebranded as Machine Learning Engineers to build and deploy machine learning models. This is what has lead to Andreessen Horowitz's Data Infrastructure 2.0, a confusing maze of tools and services that require huge(ly redundant) teams to use and maintain.




