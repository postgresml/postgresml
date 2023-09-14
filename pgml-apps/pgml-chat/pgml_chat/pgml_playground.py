from pgml import Collection, Builtins, Pipeline
import asyncio
from rich import print
async def main():
    collection = Collection("pgml_chat_all_docs_4_chat_history")
    builtins = Builtins()
    query = """SELECT metadata->>'role' as role, text as content from %s.documents
            WHERE metadata @> '{\"interface\" : \"cli\"}'::JSONB 
            AND metadata @> '{\"role\" : \"user\"}'::JSONB 
            OR metadata @> '{\"role\" : \"assistant\"}'::JSONB 
            ORDER BY metadata->>'timestamp' DESC LIMIT %d"""%("pgml_chat_readme_1_chat_history",4)
    results = await builtins.query(query).fetch_all()
    results.reverse()
    print(results)
    documents = await collection.get_documents( {
        "filter": {
            "metadata": {
                "$and" : [
                    {
                        "role": {
                            "$eq": "system"
                        }
                    },
                    {
                        "interface" : {
                            "$eq" : "slack"
                        }
                    }
                ]
            }
        }
        }
    )
    print(documents)
    pipeline = Pipeline("pgml_chat_all_docs_4_chat_history_pipeline")
    results = (
    await collection.query()
    .vector_recall("how do I use xgboost", pipeline, {"instruction": "Represent the question for retrieving supporting documents: "})
    .limit(10)
    .filter({
         "metadata": {
                "$and" : [
                    {
                        "role": {
                            "$eq": "assistant"
                        }
                    },
                    {
                        "interface" : {
                            "$eq" : "cli"
                        }
                    }
                ]
            }
    })
    .fetch_all()
    )   
    print(results)

if __name__ == "__main__":
    asyncio.run(main())
