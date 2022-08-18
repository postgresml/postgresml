from django.db import models, connection
from django.template.loader import render_to_string
from django.utils.safestring import mark_safe
from django.db.utils import ProgrammingError
from django.utils import timezone
from django.utils.html import strip_tags

import markdown

# Create your models here.
class Notebook(models.Model):
    """A notebook: collection of code, markdown, text to describe an experiment."""

    name = models.CharField(max_length=256)

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    def __str__(self):
        return self.name

    def to_markdown(self):
        """Convert the notebook to markdown so it's easily sharable."""
        result = []
        for cell in self.notebookcell_set.filter(deleted_at__isnull=True).order_by("cell_number"):
            result.append(cell.markdown())
        return "\n\n".join(result)

    def reset(self):
        """Reset all executable fields in the notebook so the user
        can play themm one at a time."""
        self.notebookcell_set.filter(cell_type=NotebookCell.SQL).update(rendering=None, execution_time=None)


class NotebookCell(models.Model):
    """A single executable cell in the notebook,
    e.g. text, markdown, code, etc."""

    MARKDOWN = 1
    PLAIN_TEXT = 2
    SQL = 3
    EMPTY = 4
    HTML = 5

    notebook = models.ForeignKey(Notebook, on_delete=models.CASCADE)
    cell_type = models.IntegerField(
        choices=(
            (
                MARKDOWN,
                "Markdown",
            ),
            (PLAIN_TEXT, "Plain text"),
            (
                SQL,
                "SQL",
            ),
            (EMPTY, "Empty"),
        )
    )
    contents = models.TextField(null=True, blank=True)
    rendering = models.TextField(null=True, blank=True)
    execution_time = models.DurationField(null=True, blank=True)
    cell_number = models.IntegerField(default=1)
    version = models.IntegerField(default=1)
    deleted_at = models.DateTimeField(null=True, blank=True)

    @property
    def html(self):
        """HTML rendering of the cell."""
        if self.rendering:
            return mark_safe(self.rendering)
        else:
            return self.rendering

    def render(self):
        """Execute the cell and save the result."""
        if self.cell_type == NotebookCell.SQL:
            execution_start = timezone.now()

            with connection.cursor() as cursor:
                try:
                    cursor.execute(self.contents)

                    if cursor.description:
                        columns = [col[0] for col in cursor.description]
                        rows = cursor.fetchall()

                        result = render_to_string(
                            "notebooks/sql.html",
                            {
                                "columns": columns,
                                "rows": rows,
                            },
                        )
                    else:
                        # Not an error, but the formatting is helpful.
                        result = render_to_string("notebooks/sql_error.html", {"error": str(cursor.statusmessage)})
                except Exception as e:
                    result = render_to_string(
                        "notebooks/sql_error.html",
                        {
                            "error": str(e),
                        },
                    )
            self.rendering = result
            self.execution_time = timezone.now() - execution_start

        elif self.cell_type == NotebookCell.MARKDOWN:
            rendering = markdown.markdown(self.contents, extensions=["extra"])

            self.rendering = '<article class="markdown-body">' + rendering + "</article>"

        elif self.cell_type == NotebookCell.PLAIN_TEXT:
            self.rendering = self.contents

        elif self.cell_type == NotebookCell.EMPTY:
            self.rendering = self.contents

        self.save()

    def markdown(self):
        """Render the cell back as markdown."""
        if self.cell_type == NotebookCell.SQL:
            return render_to_string(
                "notebooks/sql_markdown.txt", {"text": self.contents.replace(r"%%sql", "").strip()}
            )
        else:
            return self.contents

    @property
    def code(self):
        """Is this cell executable code or plain text/markdown?"""
        return self.cell_type == NotebookCell.SQL

    def __str__(self):
        return f"{self.notebook} - {self.pk}"
