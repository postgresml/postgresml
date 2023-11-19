# Discord Bot using pgml, Langchain, Instructor-xl, and Falcon 7B

In this tutorial, we will build a Discord bot that can use markdown files to help answer user inquiries. We will ingest the files, convert their contents into vector embeddings, and save them to Postgres. After indexing the data, the bot will query the collection to retrieve the documents that are most likely to answer the user's question. Then, we will use a simple SQL query utilizing PostgresML to retrieve a completion from the open source Falcon-7B-Instruct text generation model. Finally, we will return this completion to the user in the Discord channel. We will be using the [pgml python SDK](https://pypi.org/project/pgml/) to simplify the process.

In this project, we will be working with three files:

1. `./ingest.ipynb` - Jupyter notebook you can run to ingest your data into the bot.
2. `./bot.py` - The bot itself
3. `./start.py` - File that starts the bot.

## Step 1: Create a Bot and Set Up Your Environment

First, we will ensure that we have all the necessary environment variables set.

Make a copy of the `.env.template` file and name it `.env`, and ensure it is located in the root directory.

Now, we will go through each of the variables and set them to the appropriate values.

To create a Discord bot, you will need to create a Discord bot account and get a token. You can follow the tutorial on how to do that [here](https://discordpy.readthedocs.io/en/stable/discord.html). After going through this tutorial, you should have a bot created and added to your server. You should also have a token for your bot. Set this token to the variable `DISCORD_TOKEN` in your .env file.

Next, set the name of the Discord channel you would like the bot to listen to. Set this to the variable `DISCORD_CHANNEL` in your .env file.

We will be using the pgml Python SDK to create, store, and query our vectors. So, if you don't already have an account there, you can create one here: https://postgresml.org/. You can select the free serverless option and will be given a connection string. Set this connection string to the variable `pgml_CONNECTION_STR` in your .env file.

Next, you will want to add the markdown files you would like to use into the `./content` folder. Set the path to this folder to the variable `CONTENT_PATH` in your .env file.

Now that our project is set up, we can start working on our bot.

## Step 2: Ingest Your Data

Now we will ingest our markdown data into the bot.

Open and run the cells in the `./ingest.ipynb` notebook. If you have set all of your environment variables correctly, and put your markdown files into your markdown folder correctly, you should be able to run the notebook without any errors, and your bot will now have access to your data.

Let's take a look at what is happening in the notebook.

1. We load in the markdown files from the path we passed in, using Langchain's document loader.
2. We convert this array of documents to an array of dictionaries in the format expected by the pgml SDK.

```
docs = [{text: 'foo'}, {text: 'bar'}, ...]
```

1. We create a pgml collection upserting those documents into a collection.

```
collection = pgml.create_or_get_collection(collection_name)
collection.upsert_documents(docs)
```

1. We chunk those documents into smaller sizes and embed those chunks using the Instructor-XL model.

```
collection.generate_chunks()
collection.generate_embeddings()
```

Now that our data is properly indexed, we can start our bot server to handle incoming requests, using the data we just ingested to help answer questions.

## Step 3: Run Your Bot

For our bot server, we are using the popular library [discord.py](https://discordpy.readthedocs.io/en/stable/).

To start the bot server, you can run the following command in your terminal:

```
python start.py
```

If everything was set up correctly in earlier steps, your bot should be fully functional.

But since it's good to know how things are working, let's take a look at the code.

In the `start.py` file, you will see the following code:

```
# get environment variables

pg_connection_string = os.getenv("pgml_CONNECTION_STR")

# ...

## initialize bot

pgml_bot = Bot(conninfo=pg_connection_string)

## start discord bot

pgml_bot.start(collection_name, discord_token)
```

This code will initialize the bot class with your PostgreSQL connection string and then start the Discord bot with the collection name, from which you previously saved your data in the previous step, and Discord token.

If we look in the `.start` method, we will see that we execute `.run` on the `discord_client` which has been initialized.

We also declared the `on_message` function that is called when a message is sent in the Discord server.

When a message is handled by this `on_message` function, we do a few things:

1. Using the pgml SDK, we run:

```
collection.vector_search(
    query,
    top_k=3,
    model_id=2,
    splitter_id=2,
    query_parameters={"instruction": "Represent the question for retrieving supporting documents: "},
)
```

This is going to return the top 3 documents that are most similar to the user's message.

2. We then concatenate the text of those documents into a single string and add it to our prompt text, which looks like:

```
Answer the question as truthfully as possible using the provided text, and if the answer is not contained within the text below, say "I don't know!"

Context:
{context}

QUESTION<<{message_content}
ANSWER<<
```

3. Now that we have our prompt ready, we can make a Falcon completion. We will get this completion by executing a SQL query that uses `pgml.transform` function.

```
async def run_transform_sql(self, context, message_content):
    prompt = self.prepare_prompt(context, message_content)
    sql_query = """SELECT pgml.transform(
    task => '{
        "model": "tiiuae/falcon-7b-instruct",
        "device_map": "auto",
        "torch_dtype": "bfloat16",
        "trust_remote_code": true
        }'::JSONB,
        args => '{
        "max_new_tokens": 100
        }'::JSONB,
        inputs => ARRAY[%s]
        ) AS result"""
    sql_params = (prompt,)
    return await self.run_query(sql_query, sql_params)

```

4. Now that we have the response from Falcon, we need to clean the response text up a bit before returning the bot's answer. Since the completion text includes the original prompt, we will remove that from the generated text in the `prepare_response` function.
5. Finally, we will send the response back to the Discord channel.

## Final Remarks

At this point, you should have a functioning Discord bot that can answer questions based on the markdown files you have provided, using fully open-source tools.

If you have any questions or would like to chat, you can join us on the [PostgresML Discord](https://discord.gg/DmyJP3qJ7U)
