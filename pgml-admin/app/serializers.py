from rest_framework import serializers

from app.models import Project, Snapshot, Model, Deployment


class ProjectSerializer(serializers.ModelSerializer):
    class Meta:
        model = Project
        fields = "__all__"


class SnapshotSerializer(serializers.ModelSerializer):
    class Meta:
        model = Snapshot
        fields = [
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


class DeploymentSerializer(serializers.ModelSerializer):
    class Meta:
        model = Deployment
        fields = "__all__"
        depth = 1


class NewProjectSerializer(serializers.Serializer):
    project_name = serializers.CharField()
    objective = serializers.CharField()
    algorithms = serializers.ListSerializer(child=serializers.CharField())
    targets = serializers.ListSerializer(child=serializers.CharField())
    relation_name = serializers.CharField()
