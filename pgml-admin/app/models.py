from django.db import models, connection


class Project(models.Model):
    name = models.TextField()
    objective = models.TextField()
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = '"pgml"."projects"'
        managed = False

    def models(self):
        return Model.objects.filter(project=self)


class Snapshot(models.Model):
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

    def sample(self, limit=1000):
        with connection.cursor() as cursor:
            cursor.execute(f"SELECT * FROM {self.relation_name} LIMIT %s", [limit])
            columns = [col[0] for col in cursor.description]
            return [dict(zip(columns, row)) for row in cursor.fetchall()]

    @property
    def samples(self):
        return self.analysis[self.y_column_name + "_count"]

    @property
    def y_column_type(self):
        return self.columns[self.y_column_name]

    @property
    def table_size(self):
        with connection.cursor() as cursor:
            cursor.execute(f"SELECT pg_size_pretty(pg_total_relation_size(%s))", [self.relation_name])
            return cursor.fetchone()[0]

    @property
    def feature_size(self):
        return len(self.columns) - 1

class Model(models.Model):
    """A trained machine learning model."""

    project = models.ForeignKey(Project, on_delete=models.CASCADE)
    snapshot = models.ForeignKey(Snapshot, on_delete=models.CASCADE)
    algorithm_name = models.TextField()
    hyperparams = models.JSONField()
    status = models.TextField()
    metrics = models.JSONField(null=True)
    pickle = models.BinaryField(null=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        db_table = '"pgml"."models"'
        managed = False


class Deployment(models.Model):
    project = models.ForeignKey(Project, on_delete=models.CASCADE)
    model = models.ForeignKey(Model, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = '"pgml"."deployments"'
        managed = False
