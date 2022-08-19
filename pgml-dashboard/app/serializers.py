from rest_framework import serializers

from app.models import Project, Snapshot, Model, Deployment


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


class ModelSerializer(serializers.ModelSerializer):
    snapshot = SnapshotSerializer()

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
            "key_metric_name",
            "key_metric_display_name",
            "task",
            "created_at",
            "updated_at",
            "models",
        ]


class DeploymentSerializer(serializers.ModelSerializer):
    class Meta:
        model = Deployment
        fields = "__all__"
        depth = 1


class NewProjectSerializer(serializers.Serializer):
    project_name = serializers.CharField()
    task = serializers.CharField(required=False)
    relation_name = serializers.CharField(required=False)
    y_column_name = serializers.ListSerializer(child=serializers.CharField(), required=False)
    algorithms = serializers.ListSerializer(child=serializers.CharField())


class NewSnapshotSerializer(serializers.Serializer):
    relation_name = serializers.CharField()
    y_column_name = serializers.ListSerializer(child=serializers.CharField())


class RequestSerializer(serializers.Serializer):
    id = serializers.IntegerField()
    path = serializers.CharField()
    response = serializers.IntegerField()
    time = serializers.DateTimeField()
    ip = serializers.IPAddressField()
    user_agent = serializers.CharField()
