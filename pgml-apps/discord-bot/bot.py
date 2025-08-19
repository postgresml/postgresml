# We start by importing necessary packages.
import discord
from psycopg_pool import AsyncConnectionPool
from pgml import Database
from langchain.document_loaders import DirectoryLoader


# Create the Bot class
class Bot:
    # Initialize the Bot with connection info and set up a database connection pool
    def __init__(self, conninfo: str):
        self.conninfo = conninfo
        self.pool = AsyncConnectionPool(conninfo)
        self.pgml = Database(conninfo)

    # Initializes the Discord client with certain intents
    def init_discord_client(self):
        intents = discord.Intents.default()
        intents.message_content = True
        return discord.Client(intents=intents)

    # Ingests data from a directory, process it and register models, text splitters and generate chunks and embeddings
    async def ingest(self, path: str, collection_name: str):
        docs = await self.load_documents(path)
        content = await self.create_content(docs)
        collection = await self.pgml.create_or_get_collection(collection_name)
        await collection.upsert_documents(content)
        embedding_model_id = await self.register_embedding_model(collection)
        splitter_id = self.register_text_splitter(collection)
        await collection.generate_chunks(splitter_id=splitter_id)
        await collection.generate_embeddings(
            model_id=embedding_model_id, splitter_id=splitter_id
        )

    async def create_or_get_collection(self, collection_name: str):
        return await self.pgml.create_or_get_collection(collection_name)

    # Loads markdown documents from a directory
    async def load_documents(self, path: str):
        print(f"Loading documents from {path}")
        loader = DirectoryLoader(path, glob="*.md")
        docs = loader.load()
        print(f"Loaded {len(docs)} documents")
        return docs

    # Prepare content by iterating over each document
    async def create_content(self, docs):
        return [
            {"text": doc.page_content, "source": doc.metadata["source"]} for doc in docs
        ]

    # Register an embedding model to the collection
    async def register_embedding_model(self, collection):
        embedding_model_id = await collection.register_model(
            model_name="hkunlp/instructor-xl",
            model_params={"instruction": "Represent the document for retrieval: "},
        )
        return embedding_model_id

    # Register a text splitter to the collection
    async def register_text_splitter(
        self, collection, chunk_size: int = 1500, chunk_overlap: int = 40
    ):
        splitter_id = await collection.register_text_splitter(
            splitter_name="RecursiveCharacterTextSplitter",
            splitter_params={"chunk_size": chunk_size, "chunk_overlap": chunk_overlap},
        )
        return splitter_id

    # Run an SQL query and return the result
    async def run_query(self, statement: str, sql_params: tuple = None):
        conn = await self.pool.getconn()
        cur = conn.cursor()
        try:
            await cur.execute(statement, sql_params)
            return cur.fetchone()
        except Exception as e:
            print(e)
        finally:
            cur.close()

    # Query a collection with a string and return vector search results
    async def query_collection(self, collection_name: str, query: str):
        collection = await self.pgml.create_or_get_collection(collection_name)
        return collection.vector_search(
            query,
            top_k=3,
            model_id=2,
            splitter_id=2,
            query_parameters={
                "instruction": "Represent the question for retrieving supporting documents: "
            },
        )

    # Start the Discord bot, listen to messages in 'bot-testing' channel and handle the messages
    async def start(self, collection_name: str, discord_token: str, channel_name: str):
        self.discord_token = discord_token
        self.discord_client = self.init_discord_client()

        @self.discord_client.event
        async def on_ready():
            print(f"We have logged in as {self.discord_client.user}")

        @self.discord_client.event
        async def on_message(message):
            print(f"Message from {message.author}: {message.content}")

            if (
                message.author != self.discord_client.user
                and message.channel.name == channel_name
            ):
                await self.handle_message(collection_name, message)

        self.discord_client.run(self.discord_token)

    # Handle incoming messages, perform a search on the collection, and respond with a generated answer
    async def handle_message(self, collection_name, message):
        print(f"Message from {message.author}: {message.content}")
        print("Searching the vector database")
        res = await self.query_collection(collection_name, message.content)
        print(f"Found {len(res)} results")
        context = await self.build_context(res)
        print("Running Completion query")
        completion = await self.run_transform_sql(context, message.content)
        print("Preparing response")
        response = self.prepare_response(completion, context, message.content)
        print("Sending response")
        await message.channel.send(response)

    # Build the context for the message from search results
    async def build_context(self, res):
        return "\n".join([f'{r["chunk"]}' for r in res])

    # Run a SQL function 'pgml.transform' to get a generated answer for the message
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
                                "max_new_tokens": 200
                            }'::JSONB,
                            inputs => ARRAY[%s]
                        ) AS result"""
        sql_params = (prompt,)
        return await self.run_query(sql_query, sql_params)

    # Prepare the prompt to be used in the SQL function
    def prepare_prompt(self, context, message_content):
        return f"""Answer the question as truthfully as possible using the provided text, and if the answer is not contained within the text below, say "I don't know my lord!"

Context:
{context}
QUESTION<<{message_content}>>
ANSWER<<"""

    # Prepare the bot's response by removing the original prompt from the generated text
    def prepare_response(self, completion, context, message_content):
        generated_text = completion[0][0][0]["generated_text"]
        return generated_text.replace(self.prepare_prompt(context, message_content), "")
