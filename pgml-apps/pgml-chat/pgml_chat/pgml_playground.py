from pgml import Collection, Builtins, Pipeline
import asyncio
from rich import print

    # chat_history_user_messages = await chat_collection.query().vector_recall(user_input, chat_history_pipeline, query_params)(
    #     {   
    #         "limit" : chat_history,
    #         "filter": {
    #             "metadata": {
    #                 "$and": [
    #                     {{"role": {"$eq": "user"}},
    #                     {{"interface": {"$eq": chat_interface}},
    #                 ]
    #             }
    #         },
    #     }
    # ).fetch_all()

    # chat_history_assistant_messages = await chat_collection.query().vector_recall(user_input, chat_history_pipeline, query_params)(
    #     {   
    #         "limit" : chat_history,
    #         "filter": {
    #             "metadata": {
    #                 "$and": [
    #                     {"role": {"$eq": "assistant"}},
    #                     {"interface": {"$eq": chat_interface}},
    #                 ]
    #             }
    #         },
    #     }
    # ).fetch_all()


async def main():
    collection = Collection("pgml_chat_all_docs_4_chat_history")
    builtins = Builtins()
    query = """SELECT metadata->>'role' as role, text as content from %s.documents
            WHERE metadata @> '{\"interface\" : \"cli\"}'::JSONB 
            AND metadata @> '{\"role\" : \"user\"}'::JSONB 
            OR metadata @> '{\"role\" : \"assistant\"}'::JSONB 
            ORDER BY metadata->>'timestamp' DESC LIMIT %d""" % (
        "pgml_chat_readme_1_chat_history",
        4,
    )
    results = await builtins.query(query).fetch_all()
    results.reverse()
    # print(results)
    documents = await collection.get_documents(
        {   "limit": 3,
            "order_by": {"timestamp": "desc"},
            "filter": {
                "metadata": {
                    "$and": [
                        # {"role": {"$eq": "assistant"}},
                        {"interface": {"$eq": "cli"}},
                    ]
                }
            }
        }
    )
    print(documents)
    pipeline = Pipeline("pgml_chat_all_docs_4_chat_history_pipeline")
    chat_history_user_messages = (
        await collection.query()
        .vector_recall(
            "how do I use xgboost",
            pipeline,
            {
                "instruction": "Represent the question for retrieving supporting documents: "
            },
        )
        .limit(2)
        .filter(
            {
                "metadata": {
                    "$and": [
                        {"role": {"$eq": "user"}},
                        {"interface": {"$eq": "discord"}},
                    ]
                }
            }
        )
        .fetch_all()
    )

    # print(chat_history_user_messages)

    results = (
        await collection.query()
        .vector_recall(
            "PostgresML on your Ubuntu machine",
            pipeline,
            {
                "instruction": "Represent the question for retrieving supporting documents: "
            },
        )
        .limit(10)
        .filter(
            {
                "metadata": {
                    "$and": [
                        {"role": {"$eq": "assistant"}},
                        {"interface": {"$eq": "cli"}},
                    ]
                }
            }
        )
        .fetch_all()
    )
    # print(results)

    # llama2-7b-chat


if __name__ == "__main__":
    asyncio.run(main())
