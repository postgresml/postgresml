import djclick as click
import os
from django.db import transaction

from notebooks.models import Notebook, NotebookCell


@click.command()
@click.option("--path", help="Path to the Markdown file", required=True)
@click.option("--name", help="The name of the new notebook", required=True)
@transaction.atomic
def command(path, name):
    """Convert Markdown tutorials to Notebook tutorials."""
    cell = []
    cell_number = 1

    notebook = Notebook.objects.create(
        name=name,
    )

    with open(path) as f:
        for cell in f:
            # Code starts
            if cell.startswith("```sql"):
                cell = NotebookCell.objects.create(
                    contents="".join(cell),
                    cell_type=NotebookCell.MARKDOWN,
                    notebook=notebook,
                    cell_number=cell_number,
                )
                cell_number += 1
                cell.clear()
            # Code ends
            elif cell.startswith("```"):
                cell = NotebookCell.objects.create(
                    contents="%%sql\n" + "".join(cell),
                    cell_type=NotebookCell.SQL,
                    cell_number=cell_number,
                    notebook=notebook,
                )
                cell_number += 1
                cell.clear()
            # Markdown text
            else:
                cell.append(cell)

    # Whatever is left in the buffer
    if cell:
        cell = NotebookCell.objects.create(
            contents="".join(cell),
            cell_number=cell_number,
            notebook=notebook,
            cell_type=NotebookCell.MARKDOWN,
        )
