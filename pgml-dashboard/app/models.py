from django.db import models, connection, transaction
from django.template.loader import render_to_string
from django.utils.safestring import mark_safe
from django.db.utils import ProgrammingError
from django.utils import timezone
from django.utils.html import strip_tags

import markdown
import codecs
import csv


class Project(models.Model):
    name = models.TextField()
    task = models.TextField()
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = '"pgml"."projects"'
        managed = False

    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)
        self._current_deployment = None

    def models(self):
        return Model.objects.filter(project=self)

    @property
    def key_metric_name(self):
        if self.task in ["classification", "text-classification"]:
            return "f1"
        elif self.task == "regression":
            return "r2"
        else:
            raise f"""Unhandled task: "{self.task}" """

    @property
    def key_metric_display_name(self):
        if self.task in ["classification", "text-classification"]:
            return "F<sub>1</sub>"
        elif self.task == "regression":
            return "R<sup>2</sup>"
        else:
            raise f"""Unhandled task: "{self.task}" """

    @property
    def current_deployment(self):
        if self._current_deployment is None:
            self._current_deployment = self.deployment_set.order_by("-created_at").first()
        return self._current_deployment


class Snapshot(models.Model):
    """A point-in-time snapshot of the training dataset.

    The snapshot is taken before training to help reproduce the experiments.
    """

    relation_name = models.TextField()
    y_column_name = models.TextField()
    test_size = models.FloatField()
    test_sampling = models.TextField()
    status = models.TextField()
    columns = models.JSONField(null=True)
    analysis = models.JSONField(null=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = '"pgml"."snapshots"'
        managed = False

    def sample(self, limit=500):
        """Fetch a sample of the data from the snapshot."""
        with connection.cursor() as cursor:
            cursor.execute(f"SELECT * FROM pgml.{self.snapshot_name} LIMIT %s", [limit])
            columns = [col[0] for col in cursor.description]
            return [dict(zip(columns, row)) for row in cursor.fetchall()]

    @property
    def samples(self):
        """How many rows were used to perform the snapshot data analysis."""
        return self.analysis["samples"]

    @property
    def schema_name(self):
        if "." in self.relation_name:
            return self.relation_name.split(".")[0]
        return "public"

    @property
    def table_name(self):
        if "." in self.relation_name:
            return self.relation_name.split(".")[1]
        return self.relation_name

    @property
    def table_type(self):
        with connection.cursor() as cursor:
            cursor.execute(
                f"SELECT pg_size_pretty(pg_total_relation_size(%s))",
                [self.snapshot_name],
            )
            return cursor.fetchone()[0]

    @property
    def table_size(self):
        """How big is the snapshot according to Postgres."""
        try:
            with connection.cursor() as cursor:
                cursor.execute(
                    f"SELECT pg_size_pretty(pg_total_relation_size(%s))",
                    [self.snapshot_name],
                )
                return cursor.fetchone()[0]
        except:
            return 0

    @property
    def feature_size(self):
        """How many features does the dataset contain."""
        return len(self.columns) - 1

    @property
    def snapshot_name(self):
        return f"snapshot_{self.id}"


class Model(models.Model):
    """A trained machine learning model."""

    project = models.ForeignKey(Project, on_delete=models.CASCADE)
    snapshot = models.ForeignKey(Snapshot, on_delete=models.CASCADE)
    algorithm = models.TextField()
    runtime = models.TextField()
    hyperparams = models.JSONField()
    status = models.TextField()
    search = models.TextField()
    search_params = models.JSONField()
    search_args = models.JSONField()
    metrics = models.JSONField(null=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = '"pgml"."models"'
        managed = False

    @property
    def key_metric(self):
        return self.metrics[self.project.key_metric_name] or float("nan")

    def live(self):
        last_deployment = Deployment.objects.filter(project=self.project).last()
        return last_deployment.model.pk == self.pk


class Deployment(models.Model):
    project = models.ForeignKey(Project, on_delete=models.CASCADE)
    model = models.ForeignKey(Model, on_delete=models.CASCADE)
    strategy = models.TextField()
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = '"pgml"."deployments"'
        managed = False

    @property
    def human_readable_strategy(self):
        return self.strategy.replace("_", " ")


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

            results = []
            with connection.cursor() as cursor:
                queries = self.contents.split(";\n")  # Eh.
                for query in queries:
                    try:
                        cursor.execute(query)

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
                            results.append(result)
                        else:
                            # Not an error, but the formatting is helpful.
                            result = render_to_string("notebooks/sql_error.html", {"error": str(cursor.statusmessage)})
                            results.append(result)
                    except Exception as e:
                        result = render_to_string(
                            "notebooks/sql_error.html",
                            {
                                "error": str(e),
                            },
                        )
                        results.append(result)
            self.rendering = "\n".join(results)
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


class UploadedData(models.Model):
    """Data uploaded by the user through the dashboard."""

    file_type = models.IntegerField(
        choices=(
            (
                1,
                "CSV",
            ),
            (2, "JSON"),
        ),
        default=1,
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    def create_table(self, file, has_header=False):
        if file.content_type == "text/csv":
            reader = csv.reader(codecs.iterdecode(file, "utf-8"))
            headers = next(reader)

            if has_header:
                columns = ", ".join(map(lambda x: f"{x.replace(' ', '_').lower()} TEXT", headers))
            else:
                columns = ", ".join(map(lambda x: f"column_{x} TEXT", range(len(headers))))

            with transaction.atomic():
                sql = f"CREATE TABLE data_{self.pk} (" + columns + ")"

                with connection.cursor() as cursor:
                    cursor.execute(sql)
                    file.seek(0)
                    cursor.copy_expert(f"COPY data_{self.pk} FROM STDIN CSV {'HEADER' if has_header else ''}", file)
