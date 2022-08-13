import djclick as click
import os
from django.db import transaction

from notebooks.models import Notebook, NotebookLine


@click.command()
@click.option("--path", help="Path to the Markdown file", required=True)
@click.option("--name", help="The name of the new notebook", required=True)
@transaction.atomic
def command(path, name):
    """Convert Markdown tutorials to Notebook tutorials."""
    cell = []
    line_number = 1

    notebook = Notebook.objects.create(
        name=name,
    )

    with open(path) as f:
        for line in f:
            # Code starts
            if line.startswith("```sql"):
                line = NotebookLine.objects.create(
                    contents="".join(cell),
                    line_type=NotebookLine.MARKDOWN,
                    notebook=notebook,
                    line_number=line_number,
                )

                line_number += 1

                cell.clear()
            # Code ends
            elif line.startswith("```"):
                line = NotebookLine.objects.create(
                    contents="%%sql\n" + "".join(cell),
                    line_type=NotebookLine.SQL,
                    line_number=line_number,
                    notebook=notebook,
                )
                cell.clear()
            # Markdown text
            else:
                cell.append(line)

    # Whatever is left in the buffer
    if cell:
        line = NotebookLine.objects.create(
            contents="".join(cell),
            line_number=line_number,
            notebook=notebook,
            line_type=NotebookLine.MARKDOWN,
        )
