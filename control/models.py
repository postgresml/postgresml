from django.db import models


class Project(models.Model):
    name = models.TextField()
    objective = models.TextField()
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    class Meta:
        db_table = '"pgml"."projects"'

class Snapshot(models.Model):
    relation_name = models.TextField()
    y_column_name = models.TextField()
    test_size = models.FloatField()
    test_sampling = models.TextField()
    status  = models.TextField()
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    class Meta:
        db_table = '"pgml"."snapshots"'

class Model(models.Model):
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

class Deployment(models.Model):
    project = models.ForeignKey(Project, on_delete=models.CASCADE)
    model = models.ForeignKey(Model, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)
    class Meta:
        db_table = '"pgml"."deployments"'
