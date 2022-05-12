from rest_framework import serializers

from app.models import Project, Snapshot, Model, Deployment


class ModelSerializer(serializers.ModelSerializer):
    class Meta:
        model = Model
        fields = [
            "algorithm_name",
            "hyperparams",
            "status",
            "metrics",
            "key_metric",
            "live",
            "snapshot",
            "created_at",
            "updated_at",
        ]
        depth = 1

class ProjectSerializer(serializers.ModelSerializer):
    models = ModelSerializer(many=True)

    class Meta:
        model = Project
        fields = [
            "id",
            "name",
            "objective",
            "created_at",
            "updated_at",
            "models",
        ]


class SnapshotSerializer(serializers.ModelSerializer):
    y_column_name = serializers.ListSerializer(child=serializers.CharField())

    class Meta:
        model = Snapshot
        fields = [
            "id",
            "y_column_name",
            "test_size",
            "test_sampling",
            "status",
            "columns",
            "analysis",
            "sample",
            "samples",
            "table_size",
            "feature_size",
            "created_at",
            "updated_at",
        ]


class DeploymentSerializer(serializers.ModelSerializer):
    class Meta:
        model = Deployment
        fields = "__all__"
        depth = 1


class NewProjectSerializer(serializers.Serializer):
    project_name = serializers.CharField()
    objective = serializers.CharField()
    snapshot_id = serializers.IntegerField()
    algorithms = serializers.ListSerializer(child=serializers.CharField())


class NewSnapshotSerializer(serializers.Serializer):
    relation_name = serializers.CharField()
    y_column_name = serializers.ListSerializer(child=serializers.CharField())
