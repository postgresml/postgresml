from django.db import models, connection

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
        with connection.cursor() as cursor:
            cursor.execute(
                f"SELECT pg_size_pretty(pg_total_relation_size(%s))",
                [self.snapshot_name],
            )
            return cursor.fetchone()[0]

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
    algorithm_name = models.TextField()
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
        return self.metrics[self.project.key_metric_name]

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
