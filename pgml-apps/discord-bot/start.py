# Importing libraries
from bot import Bot
from dotenv import load_dotenv
import os
load_dotenv()


# get environment variables
pg_connection_string = os.getenv("PGML_CONNECTION_STR")
collection_name = os.getenv("COLLECTION_NAME")
markdown_folder_path = os.getenv("CONTENT_PATH")
discord_token = os.getenv("DISCORD_TOKEN")
channel_name = os.getenv("DISCORD_CHANNEL")

## initialize bot
pgml_bot = Bot(conninfo=pg_connection_string)

## start discord bot
pgml_bot.start(collection_name, discord_token, channel_name)