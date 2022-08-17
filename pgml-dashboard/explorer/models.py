from django.db import models, connection
from django.template.loader import render_to_string
from django.utils import timezone


class Query(models.Model):
    """A query executed using the explorer."""

    contents = models.TextField()
    rendering = models.TextField(null=True, blank=True)
    execution_time = models.DurationField(null=True, blank=True)

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    def html(self):
        if self.rendering:
            return self.rendering
        else:
            with connection.cursor() as cursor:
                start_time = timezone.now()
                try:
                    cursor.execute(self.contents)
                    columns = [col[0] for col in cursor.description]
                    rows = cursor.fetchall()

                    self.rendering = render_to_string(
                        "explorer/query.html",
                        {
                            "columns": columns,
                            "rows": rows,
                        },
                    )
                except Exception as e:
                    self.rendering = render_to_string(
                        "explorer/error.html",
                        {
                            "error": str(e),
                        },
                    )
            self.execution_time = timezone.now() - start_time
            self.save()
            return self.rendering

    def __str__(self):
        return self.contents[:25]

    class Meta:
        verbose_name_plural = "queries"


class Table:
    def __init__(self, name):
        self.name = name

    def __str__(self):
        return self.name


class Schema:
    def __init__(self, name, tables=[]):
        self.name = name
        self.tables = tables

    def __str__(self):
        return self.name
