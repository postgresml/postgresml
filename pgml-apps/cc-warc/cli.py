from warcio.archiveiterator import ArchiveIterator
import click
import asyncio
import requests
from pgml import Collection, Pipeline
from config import DATABASE_URL

collection = Collection("warc", DATABASE_URL)
pipeline = Pipeline("warc_search", {
	"body": {
		"splitter": {"model": "recursive_character"},
		"semantic_search": {
			"model": "mixedbread-ai/mxbai-embed-large-v1",
		}
	}
})

async def ingest(paths, limit=500):
	await collection.add_pipeline(pipeline)
	with open(paths) as f:
		for path in f:
			req = requests.get("https://data.commoncrawl.org/%s" % path.strip(), stream=True)
			batch = []
			for record in ArchiveIterator(req.raw, arc2warc=True):
				document = {
					"id": record.rec_headers.get_header("WARC-Target-URI"),
					"body": record.content_stream().read().decode("utf-8")
				}
				print(document)
				batch.append(document)
				if len(batch) == limit:
					exit(1)
					# await collection.upsert_documents(batch)
					batch = []


@click.command()
@click.option("--path", help="Path to the WET paths file.", default="paths.txt")
@click.option("--limit", default=5, help="How many files to download and ingest.")
def cli(path, limit):
	asyncio.run(ingest(path, limit))


if __name__ == "__main__":
	cli()
