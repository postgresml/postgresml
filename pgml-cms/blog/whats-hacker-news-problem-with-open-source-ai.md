---
description: >-
  Open source AI is not the future. It’s here, now. Hacker News has spent the last 24 hours debating if Meta’s Llama models are really “open source” rather than talking about the ramifications of its launch.
featured: false
tags: [engineering]
image: ".gitbook/assets/keep-ai-open.png"
---

# What’s Hacker News’ problem with open source AI

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Montana Low

July 24, 2024

Open source AI is not the future. It’s here, now. Hacker News has spent the [last 24 hours debating](https://news.ycombinator.com/item?id=41046773) if Meta’s Llama models are really “open source” rather than talking about the ramifications of its launch. They similarly debate what “AI” is. Open source AI is important, not because of some pedantic definition by some pseudo-official body like OSI, it’s important because of the power and incentive structures that pervade our society.

Open source AI is not just about LLMs and licenses. The term is more useful when it is used to describe the full stack required to create value for end users. LLMs alone are not enough to create AI, and training them is a cost without an economically defensible moat. That cost is going to increase and the value is going to approach zero as they are commoditized. Value creation happens as part of a larger process.

People on Hacker News should be discussing that process, since it involves a complete software application, which is built with hundreds of linked open source libraries running across many machines, often in different physical regions. Software engineers need to grapple with the centuries-old engineering questions of how we efficiently, reliably and safely manage increasing complexity while working with more sophisticated methods.

## Please move beyond pedantic definitions and personality cults

Fanboys and haters are no more helpful in this discussion than they are in politics. It seems lost on many that Mark Zuckerberg may not be the villain in this story, and Sam Altman may not be the hero. They are both CEOs of powerful companies that are trying to shape the technology that has the most potential to change our society since the internet was created. What we also know is that Mark has _consistently_ rationalized Meta’s interest in open source AI, and I trust him to look after _his_ interests. Sam has _inconsistently_ rationalized OpenAIs interest in AI, and I do not trust him to look after _all of humanity's_ interests.

Llama is an important piece in the open source AI ecosystem.

- You are free to run it on your laptop or in your datacenter, unless you have 700,000,000 users. Many open source licenses come with restrictions on use and this is a generous one.
- You are free to modify it with fine-tuning, quantization, cut-and-paste layers or any other way you want.
- You are free to understand it as much as the people who built it, since they’ve helpfully published extensive documentation and academic papers, and released the source code required to experiment with it.

Full open data has never been a standard, much less requirement, for open source or any academic publishing process. “open-weight” vs “open-source” is a distinction without a difference for most of the world.

Meta has been contributing to open source AI beyond Llama for a long time. Pytorch is the de facto industry standard for training, tuning and running models. One observation should be that there is so much more than weights or a runtime involved in value creation, that even a trillion-dollar company realizes they need the support of a larger open source community to succeed, and is willing to give those pieces away to get help. This seems like the more likely path to benefit all of humanity.

## The power of a completely open source stack

A complete open-source stack encompasses data preprocessing, model deployment, scaling, and monitoring. It’s the combination of these elements that allows for the creation of innovative, robust, and efficient AI-driven applications. Here’s why a fully open-source approach wins:

### Transparency and trust

Transparency is a cornerstone of open-source projects. When every component of the stack is open, it’s easier to understand how data is being processed, how models are being trained, and how decisions are being made. This transparency builds trust with users and stakeholders, who can be assured that the system operates as claimed, free from hidden biases or unexplained behaviors.

### Flexibility and customization

Open source tools offer unmatched flexibility. Proprietary solutions often come with limitations, either through design or licensing. With an open-source stack, you have the freedom to customize every aspect to fit your unique needs. This can lead to more innovative solutions tailored to specific problems, giving you a competitive edge.

### Cost efficiency

While the initial cost of developing an open-source AI stack may be significant, the long-term benefits far outweigh these initial investments. Proprietary solutions often come with ongoing licensing fees and usage costs that can quickly add up. An open-source stack, on the other hand, eliminates these recurring costs, providing a more sustainable and scalable solution.

### Community and collaboration

The open-source community is a powerhouse of innovation and collaboration. By leveraging a fully open-source stack, you can tap into a vast pool of knowledge, resources, and support. This community-driven approach accelerates development, as you can build on the work of others and contribute your improvements back to the community.

## The pitfalls of proprietary models
Proprietary AI models are often touted for their performance and ease of use. However, they come with several significant drawbacks:

### Lack of transparency

Proprietary models are black boxes. Without access to the underlying code, documentation or research, it’s impossible to fully understand how these models operate, leading to potential trust issues. This lack of transparency can be particularly problematic in sensitive applications where understanding model decisions is critical.

### Vendor lock-in

Relying on proprietary solutions often leads to vendor lock-in, where switching to another solution becomes prohibitively expensive or complex. This dependency can stifle innovation and limit your ability to adapt to new technologies or methodologies.

### Ethical and legal concerns

Using proprietary models can raise ethical and legal concerns, particularly regarding data privacy and usage rights. Without visibility into how models are trained and designed, there’s a risk of inadvertently violating privacy regulations or getting biased results.

## PostgresML: A comprehensive open source solution

PostgresML is an end-to-end machine learning and AI platform that exemplifies the power of a complete open source stack. PostgresML integrates machine learning capabilities directly into PostgreSQL, providing a seamless environment for data storage, feature engineering, model training, and inference.
Key advantages:

- **Integrated Environment**: PostgresML eliminates the need for complex data pipelines by integrating ML directly into the database, reducing latency and improving performance.
- **Scalability**: Leveraging PostgreSQL’s robust architecture, PostgresML can scale with your data with your models, providing enterprise-level performance and reliability.
- **Community and Ecosystem**: Built on the shoulders of giants, PostgresML benefits from the extensive PostgreSQL community and ecosystem, ensuring continuous improvement and support.

## Looking to the future

Open source AI is a healthy reversion to the industry norm. By embracing open source tools and platforms like PostgresML and Llama, we not only gain transparency, control, and cost efficiency but also foster a collaborative environment that drives innovation. As the landscape of AI continues to evolve, the benefits of open source will become even more pronounced, further solidifying its role as the backbone of modern application development.

The future of AI-driven applications lies in the adoption of a complete open source stack. It’s crucial to remember the importance of openness—not just for the sake of ideology, but for the tangible benefits it brings to our projects and society as a whole. Open source AI is here, and it’s time to harness its full potential.

