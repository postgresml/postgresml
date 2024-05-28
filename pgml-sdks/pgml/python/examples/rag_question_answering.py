from pgml import Collection, Pipeline, OpenSourceAI, init_logger
import json
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
import asyncio


init_logger()


async def main():
    load_dotenv()
    console = Console()

    # Initialize collection
    collection = Collection("squad_collection")

    # Create and add pipeline
    pipeline = Pipeline(
        "squadv1",
        {
            "text": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {"model": "Alibaba-NLP/gte-base-en-v1.5"},
            }
        },
    )
    await collection.add_pipeline(pipeline)

    # Prep documents for upserting
    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])
    documents = [
        {"id": r["id"], "text": r["context"], "title": r["title"]}
        for r in data.to_dict(orient="records")
    ]

    # Upsert documents
    await collection.upsert_documents(documents[:200])

    # Query for context
    query = "Who won more than 20 grammy awards?"
    console.print("Querying for context ...")
    start = time()
    results = await collection.vector_search(
        {"query": {"fields": {"text": {"query": query}}}, "limit": 10}, pipeline
    )
    end = time()
    console.print("\n Results for '%s' " % (query), style="bold")
    console.print(results)
    console.print("Query time = %0.3f" % (end - start))

    # Construct context from results
    chunks = [r["chunk"] for r in results]
    context = "\n\n".join(chunks)

    # Query for answer
    system_prompt = """Use the following pieces of context to answer the question at the end.
        If you don't know the answer, just say that you don't know, don't try to make up an answer.
        Use three sentences maximum and keep the answer as concise as possible.
        Always say "thanks for asking!" at the end of the answer."""
    user_prompt_template = """
    ####
    Documents
    ####
    {context}
    ###
    User: {question}
    ###
    """

    user_prompt = user_prompt_template.format(context=context, question=query)
    messages = [
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": user_prompt},
    ]

    # Using OpenSource LLMs for Chat Completion
    client = OpenSourceAI()
    chat_completion_model = "meta-llama/Meta-Llama-3-8B-Instruct"
    console.print("Generating response using %s LLM..."%chat_completion_model)
    response = client.chat_completions_create(
        model=chat_completion_model,
        messages=messages,
        temperature=0.3,
        max_tokens=256,
    )
    output = response["choices"][0]["message"]["content"]
    console.print("Answer: %s"%output)
    # Archive collection
    await collection.archive()


if __name__ == "__main__":
    asyncio.run(main())
