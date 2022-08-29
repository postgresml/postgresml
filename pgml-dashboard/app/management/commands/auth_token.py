import jwt
import djclick as click

from django.conf import settings


@click.command()
@click.option("--shard", default="0")
def command(shard):
    """Generate an auth token."""
    claim = {"dashboard": "1234", "shard": shard}

    token = jwt.encode(claim, settings.SECRET_KEY, algorithm="HS256")
    print(token)
