from pgml import Database
import os
from dotenv import load_dotenv
import asyncio
from rich.pretty import pprint
# Load .env file
load_dotenv(".env")

database_url = os.environ.get("DATABASE_URL")
db = Database(database_url)


async def main():
    prompt = (
        "Girafatron is obsessed with giraffes, the most glorious animal on the face of this Earth. Giraftron believes all other animals are irrelevant when compared to the glorious majesty of the giraffe.\nDaniel: Hello, Girafatron!\nGirafatron:",
    )
    results = await db.transform(
        task={
            "task": "text-generation",
            "model": "tiiuae/falcon-7b-instruct",
            "trust_remote_code": True,
            "torch_dtype": "bfloat16",
            "device_map": "auto",
        },
        inputs=prompt,
        args={
            "temperature": 0.7,
            "top_p": 0.9,
            "max_length" : 256
        },
    )
    pprint(results)


if __name__ == "__main__":
    asyncio.run(main())
