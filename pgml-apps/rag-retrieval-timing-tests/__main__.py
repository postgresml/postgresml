import time
import asyncio

import postgresml as pgl
import zilliz_local as zl
import pinecone_local as pl
import qdrant_local as ql
import openai_local as al
import huggingface as hf
import weaviate_local as wl

TRIAL_COUNT = 2

# The pairs we are testing with
tests = [
    {
        "name": "PostgresML",
        "vector_store": pgl,
        "rag+": True,
        "chatbot_service": al,
        "async": True,
    },
    {"name": "Weaviate", "vector_store": wl, "chatbot_service": al, "rag++": True},
    {
        "name": "Zilliz",
        "vector_store": zl,
        "embedding_service": hf,
        "chatbot_service": al,
    },
    {
        "name": "Pinecone",
        "vector_store": pl,
        "embedding_service": hf,
        "chatbot_service": al,
    },
    {
        "name": "Qdrant",
        "vector_store": ql,
        "embedding_service": hf,
        "chatbot_service": al,
    },
]


# Our documents
# We only really need to test on 2. When we search we are trying to get the first document back
documents = [
    {"id": "0", "metadata": {"text": "The hidden value is 1000"}},
    {
        "id": "1",
        "metadata": {"text": "This is just some random text"},
    },
]


def maybe_do_async(func, check_dict, *args):
    if "async" in check_dict and check_dict["async"]:
        return asyncio.run(func(*args))
    else:
        return func(*args)


def do_data_upsert(name, vector_store, **kwargs):
    print(f"Doing Data Upsert For: {name}")
    if "rag++" in kwargs or "rag+" in kwargs:
        maybe_do_async(vector_store.upsert_data, kwargs, documents)
    else:
        texts = [d["metadata"]["text"] for d in documents]
        (embeddings, time_to_embed) = kwargs["embedding_service"].get_embeddings(texts)
        maybe_do_async(vector_store.upsert_data, kwargs, documents, embeddings)
    print(f"Done Doing Data Upsert For: {name}\n")


def do_normal_rag_test(name, vector_store, **kwargs):
    print(f"Doing RAG Test For: {name}")
    query = "What is the hidden value?"
    if "rag++" in kwargs:
        (result, time_to_complete) = maybe_do_async(
            vector_store.get_llm_response, kwargs, query
        )
        time_to_embed = 0
        time_to_search = 0
    elif "rag+" in kwargs:
        time_to_embed = 0
        (context, time_to_search) = maybe_do_async(
            vector_store.do_search, kwargs, query
        )
        (result, time_to_complete) = kwargs["chatbot_service"].get_llm_response(
            query, context
        )
    else:
        (embeddings, time_to_embed) = kwargs["embedding_service"].get_embeddings(
            [query]
        )
        (context, time_to_search) = vector_store.do_search(embeddings[0])
        (result, time_to_complete) = kwargs["chatbot_service"].get_llm_response(
            query, context
        )
    print(f"\tThe LLM Said: {result}")
    time_for_retrieval = time_to_embed + time_to_search
    total_time = time_to_embed + time_to_search + time_to_complete
    print(f"Done Doing RAG Test For: {name}")
    print(f"- Time to Embed: {time_to_embed}")
    print(f"- Time to Search: {time_to_search}")
    print(f"- Total Time for Retrieval: {time_for_retrieval}")
    print(f"- Time for Chatbot Completion: {time_to_complete}")
    print(f"- Total Time Taken: {total_time}\n")
    return {
        "time_to_embed": time_to_embed,
        "time_to_search": time_to_search,
        "time_for_retrieval": time_for_retrieval,
        "time_to_complete": time_to_complete,
        "total_time": total_time,
    }


if __name__ == "__main__":
    print("----------Doing Data Setup-------------------------\n")
    for test in tests:
        do_data_upsert(**test)
    print("\n----------Done Doing Data Setup------------------\n\n")

    print("----------Doing Rag Tests-------------------------\n")
    stats = {}
    for i in range(TRIAL_COUNT):
        for test in tests:
            times = do_normal_rag_test(**test)
            if not test["name"] in stats:
                stats[test["name"]] = []
            stats[test["name"]].append(times)
    print("\n----------Done Doing Rag Tests---------------------\n")

    print("------------Final Results---------------------------\n")
    for test in tests:
        trials = stats[test["name"]]
        (
            time_to_embed,
            time_to_search,
            time_for_retrieval,
            time_to_complete,
            total_time,
        ) = [
            sum(trial[key] for trial in trials)
            for key in [
                "time_to_embed",
                "time_to_search",
                "time_for_retrieval",
                "time_to_complete",
                "total_time",
            ]
        ]
        print(f'Done Doing RAG Test For: {test["name"]}')
        print(f"- Average Time to Embed: {(time_to_embed / TRIAL_COUNT):0.4f}")
        print(f"- Average Time to Search: {(time_to_search / TRIAL_COUNT):0.4f}")
        print(
            f"- Average Total Time for Retrieval: {(time_for_retrieval / TRIAL_COUNT):0.4f}"
        )
        print(
            f"- Average Time for Chatbot Completion: {(time_to_complete / TRIAL_COUNT):0.4f}"
        )
        print(f"- Average Total Time Taken: {(total_time / TRIAL_COUNT):0.4f}\n")
