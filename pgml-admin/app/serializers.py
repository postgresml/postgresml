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
    model = ModelSerializer()

    # Don't use the snapshot serializer here on purpose, because that serializer
    # also provides the sample data which can be excessive for this specific API endpoint.

    class Meta:
        model = Deployment
        fields = "__all__"
        depth = 1
