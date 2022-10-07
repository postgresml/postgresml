from tracemalloc import Snapshot
from typing import OrderedDict

from django.db import connection
from django.shortcuts import render, get_object_or_404
from django.utils.safestring import SafeString

from rest_framework.decorators import action
from rest_framework import viewsets
from rest_framework.response import Response
from rest_framework import status

import json
from app.models import Snapshot, Project, Model
from app.serializers import SnapshotSerializer, NewSnapshotSerializer

from collections import namedtuple


def default_context(context):
    return {"topic": "snapshots", **context}


def index(request):
    snapshots = Snapshot.objects.order_by("-created_at").all()
    context = default_context({"title": "Snapshots", "snapshots": snapshots})
    return render(request, "snapshots/index.html", context)


def snapshot(request, id):
    if request.method == "GET":
        return get(request, id)


def get(request, id):
    snapshot = get_object_or_404(Snapshot, id=id)
    models = snapshot.model_set.all().prefetch_related("project")
    projects = OrderedDict()
    for model in models:
        if model.project.name in projects:
            projects[model.project.name][1].append(model)
        else:
            projects[model.project.name] = (model.project, [model])
    P = namedtuple("P", "models metric min_score max_score id")
    for project_name, stuff in projects.items():
        project = stuff[0]
        models = stuff[1]
        scores = [model.key_metric for model in models]
        projects[project_name] = P(
            sorted(models, key=lambda model: -model.key_metric),
            project.key_metric_display_name,
            0,
            max(scores),
            project.id,
        )

    try:
        samples = snapshot.sample(500)
    except:
        samples = []

    columns = OrderedDict()
    try:
        # v1.0 format
        column_names = sorted(list(snapshot.columns.keys()))
        for target in snapshot.y_column_name:
            column_names.remove(target)
            column_names.insert(0, target)
        for column_name in column_names:
            if snapshot.columns[column_name] in ["integer", "real", "boolean"]:
                sample = [sample[column_name] for sample in samples]
                # TODO reconsider boolean support, cast during snapshot?
                if snapshot.columns[column_name] == "boolean":
                    sample = [float(x) for x in sample]

                columns[column_name] = {
                    "name": column_name,
                    "type": snapshot.columns[column_name],
                    "q1": snapshot.analysis[column_name + "_p25"],
                    "median": snapshot.analysis[column_name + "_p50"],
                    "q3": snapshot.analysis[column_name + "_p75"],
                    "mean": snapshot.analysis[column_name + "_mean"],
                    "stddev": snapshot.analysis[column_name + "_stddev"],
                    "min": snapshot.analysis[column_name + "_min"],
                    "max": snapshot.analysis[column_name + "_max"],
                    "dip": snapshot.analysis[column_name + "_dip"],
                    "samples": SafeString(json.dumps(sample)),
                }
    except:
        # v2.0 format
        for column in snapshot.columns:
            column_name = column["name"]
            if "[]" not in column_name:
                sample = [sample[column_name] for sample in samples]
                if "bool" == column["pg_type"]:
                    sample = [float(x) for x in sample]

                columns[column_name] = {
                    "name": column_name,
                    "type": column["pg_type"],
                    "q1": snapshot.analysis[column_name + "_p25"],
                    "median": snapshot.analysis[column_name + "_p50"],
                    "q3": snapshot.analysis[column_name + "_p75"],
                    "mean": snapshot.analysis[column_name + "_mean"],
                    "stddev": snapshot.analysis[column_name + "_stddev"],
                    "min": snapshot.analysis[column_name + "_min"],
                    "max": snapshot.analysis[column_name + "_max"],
                    "samples": SafeString(json.dumps(sample)),
                }


    # TODO reconsider spaces in column_names, fix during snapshot?
    fixed_columns = OrderedDict()
    for column_name, values in columns.items():
        fixed_columns[column_name.replace(" ", "_")] = values
    columns = fixed_columns
    context = {
        "snapshot": snapshot,
        "features": columns,
        "projects": projects,
    }
    return render(request, "snapshots/snapshot.html", default_context(context))


class SnapshotAnalysisView(viewsets.ViewSet):

    permission_classes = []

    def list(self, request):
        if "snapshot_id" not in request.GET:
            return Response(status=status.HTTP_400_BAD_REQUEST)

        snapshot = get_object_or_404(Snapshot, id=request.GET["snapshot_id"])
        context = {
            "labels": [
                {
                    "name": column,
                    "type": snapshot.columns[column],
                    "samples": list(map(lambda x: x[column], snapshot.sample())),
                }
                for column in snapshot.y_column_name
            ],
            "features": [
                {
                    "name": column,
                    "type": snapshot.columns[column],
                    "samples": list(map(lambda x: x[column], snapshot.sample())),
                }
                for column in snapshot.columns.keys() - snapshot.y_column_name
            ],
            "model": Model.objects.filter(snapshot=snapshot, algorithm="linear").first(),
        }

        return render(request, "snapshots/analysis.html", context)


class SnapshotViewSet(viewsets.ModelViewSet):
    queryset = Snapshot.objects.all()
    serializer_class = SnapshotSerializer

    @action(detail=False, permission_classes=[], methods=["POST"])
    def snapshot(self, request):
        """Create a snapshot using pgml.snapshot"""
        serializer = NewSnapshotSerializer(data=request.data)
        if serializer.is_valid():
            with connection.cursor() as cursor:
                cursor.execute(
                    """
                    SELECT * FROM pgml.snapshot(
                        relation_name => %s,
                        y_column_name => %s
                    )
                """,
                    [
                        serializer.validated_data["relation_name"],
                        serializer.validated_data["y_column_name"],
                    ],
                )
                result = cursor.fetchone()
            snapshot = Snapshot.objects.filter(pk=result[0]).first()
            return Response(status=status.HTTP_201_CREATED, data=SnapshotSerializer(snapshot).data)
        else:
            return Response(status=status.HTTP_400_BAD_REQUEST, data=serializer.errors)
