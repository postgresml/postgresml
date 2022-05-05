from tracemalloc import Snapshot
from typing import OrderedDict
from django.shortcuts import render, get_object_or_404
from django.utils.safestring import SafeString
from django.db import connection
from rest_framework import viewsets

import json
from app.models import Snapshot, Project, InformationSchemaTable
from app.serializers import SnapshotSerializer

from collections import namedtuple


def default_context(context):
    return {"topic": "Snapshots", **context}


def index(request):
    snapshots = Snapshot.objects.order_by("-created_at").all()
    context = default_context({"title": "Snapshots", "snapshots": snapshots})
    return render(request, "snapshots/index.html", context)


def snapshot(request, id):
    if request.method == "GET":
        return get(request, id)


def get(request, id):
    snapshot = get_object_or_404(Snapshot, id=id)
    samples = snapshot.sample(500)
    columns = OrderedDict()
    column_names = sorted(list(snapshot.columns.keys()))
    for target in snapshot.y_column_name:
        column_names.remove(target)
        column_names.insert(0, target)

    for column_name in column_names:
        if snapshot.columns[column_name] in ["integer", "real"]:
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
                "samples": SafeString(json.dumps([sample[column_name] for sample in samples])),
            }

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

    context = {
        "snapshot": snapshot,
        "features": columns,
        "projects": projects,
    }
    return render(request, "snapshots/snapshot.html", default_context(context))


def preview(request):
    """Preview a small sample of the table/snapshot.

    This works on tables and views and it's not snapshot-specific.
    """
    table = request.GET.get("table", "")
    if "." in table:
        schema, table = tuple(table.split("."))
    else:
        schema, table = "public", table

    table = get_object_or_404(InformationSchemaTable, table_name=table, table_schema=schema)

    with connection.cursor() as cursor:
        # Not a SQL injection because the table name comes from the DB.
        cursor.execute(f"SELECT * FROM {table.table_name} LIMIT 10")

        colnames = [desc[0] for desc in cursor.description]
        data = cursor.fetchall()

        return render(request, "snapshots/preview.html", {
            "columns": colnames,
            "rows": data,
            "controller": "new-project", # Hardcoded for now
        })


def targets(request):
    """Preview a small sample of the table/snapshot.

    This works on tables and views and it's not snapshot-specific.
    """
    table = request.GET.get("table", "")
    if "." in table:
        schema, table = tuple(table.split("."))
    else:
        schema, table = "public", table

    print(f"'{schema}', '{table}'")

    table = get_object_or_404(InformationSchemaTable, table_name=table, table_schema=schema)
    with connection.cursor() as cursor:
        cursor.execute(f"SELECT * FROM {table.table_name} LIMIT 1")

        colnames = [desc[0] for desc in cursor.description]
        data = cursor.fetchone()

        columns = [{"name": name, "data_type": type(data[i]).__name__} for i, name in enumerate(colnames)]

        return render(request, "snapshots/targets.html", {
            "columns": columns,
            "controller": "new-project", # BUG: hardcoded
        })
    

class SnapshotViewSet(viewsets.ModelViewSet):
    queryset = Snapshot.objects.all()
    serializer_class = SnapshotSerializer
    # filterset_fields = [""]
