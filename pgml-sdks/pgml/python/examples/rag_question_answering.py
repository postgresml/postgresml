from pgml import Collection, Model, Splitter, Pipeline, Builtins, OpenSourceAI
import json
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
import asyncio


async def main():
    load_dotenv()
    console = Console()

    # Initialize collection
    collection = Collection("squad_collection")

    # Create a pipeline using the default model and splitter
    model = Model()
    splitter = Splitter()
    pipeline = Pipeline("squadv1", model, splitter)
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

    console.print("Question: %s"%query)
    console.print("Querying for context ...")

    start = time()
    results = (
        await collection.query().vector_recall(query, pipeline).limit(5).fetch_all()
    )
    end = time()
    
    #console.print("Query time = %0.3f" % (end - start))

    # Construct context from results
    context = " ".join(results[0][1].strip().split())
    context = context.replace('"', '\\"').replace("'", "''")
    console.print("Context is ready...")

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
    chat_completion_model = "HuggingFaceH4/zephyr-7b-beta"
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
