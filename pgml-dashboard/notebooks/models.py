from django.db import models, connection
from django.template.loader import render_to_string
from django.utils.safestring import mark_safe
from django.db.utils import ProgrammingError
from django.utils import timezone

import markdown

# Create your models here.
class Notebook(models.Model):
    """A notebook: collection of code, markdown, text to describe an experiment."""

    name = models.CharField(max_length=256, unique=True)

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    def __str__(self):
        return self.name


class NotebookLine(models.Model):
    """A single executable line in the notebook,
    e.g. text, markdown, code, etc."""

    MARKDOWN = 1
    PLAIN_TEXT = 2
    SQL = 3
    EMPTY = 4
    HTML = 5

    notebook = models.ForeignKey(Notebook, on_delete=models.CASCADE)
    line_type = models.IntegerField(
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
    line_number = models.IntegerField(default=1)
    version = models.IntegerField(default=1)
    deleted_at = models.DateTimeField(null=True, blank=True)

    def html(self):
        """HTML rendering of the notebook line"""
        if self.rendering is not None:
            return mark_safe(self.rendering)

        if self.line_type == NotebookLine.SQL:
            execution_start = timezone.now()

            with connection.cursor() as cursor:
                try:
                    cursor.execute(self.contents.replace(r"%%sql", ""))

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
                        # Not really an error, but the formatting is helpful
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
            self.save()

        elif self.line_type == NotebookLine.MARKDOWN:
            rendering = markdown.markdown(self.contents)

            # Make sure that multi-line code blocks presenve their multiple lines.
            # Weird that the package doesn't do that already?
            rendering = rendering.replace("<code>", "<pre><code>")
            rendering = rendering.replace("</code>", "</code></pre>")

            self.rendering = '<article class="markdown-body">' + rendering + "</article>"
            self.save()

        elif self.line_type == NotebookLine.PLAIN_TEXT:
            self.rendering = self.contents
        elif self.line_type == NotebookLine.EMPTY:
            self.rendering = self.contents

        return mark_safe(self.rendering)

    @property
    def code(self):
        return self.line_type == NotebookLine.SQL

    def __str__(self):
        return f"{self.notebook} - {self.pk}"
