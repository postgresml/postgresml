from typing import OrderedDict
from django.shortcuts import render, get_object_or_404
from django.utils.safestring import SafeString
import json
from .. import models


def default_context(context):
    return {"topic": "Snapshots", **context}


def index(request):
    snapshots = models.Snapshot.objects.all()
    context = default_context({"title": "Snapshots", "snapshots": snapshots})
    return render(request, "snapshots/index.html", context)


def snapshot(request, id):
    if request.method == "GET":
        return get(request, id)


def get(request, id):
    snapshot = get_object_or_404(models.Snapshot, id=id)
    samples = snapshot.sample(1000)
    columns = OrderedDict()
    column_names = list(snapshot.columns.keys())
    column_names.sort()
    for column_name in column_names:
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
            "samples": SafeString(json.dumps([sample[column_name] for sample in samples])),
        }

    context = {
        "snapshot": snapshot,
        "features": columns,
    }
    return render(request, "snapshots/snapshot.html", context)
