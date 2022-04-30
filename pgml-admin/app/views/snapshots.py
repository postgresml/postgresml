from tracemalloc import Snapshot
from typing import OrderedDict
from django.shortcuts import render, get_object_or_404
from django.utils.safestring import SafeString
import json
from ..models import Snapshot

from collections import namedtuple

def default_context(context):
    return {"topic": "Snapshots", **context}


def index(request):
    snapshots = Snapshot.objects.all()
    context = default_context({"title": "Snapshots", "snapshots": snapshots})
    return render(request, "snapshots/index.html", context)


def snapshot(request, id):
    if request.method == "GET":
        return get(request, id)


def get(request, id):
    snapshot = get_object_or_404(Snapshot, id=id)
    samples = snapshot.sample(500)
    columns = OrderedDict()
    column_names = list(snapshot.columns.keys())
    column_names.sort()
    column_names.remove(snapshot.y_column_name)
    column_names.insert(0, snapshot.y_column_name)
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
            "dip": snapshot.analysis[column_name + "_dip"],
            "samples": SafeString(json.dumps([sample[column_name] for sample in samples])),
        }

    models = snapshot.model_set.all().prefetch_related("project")
    projects = OrderedDict()
    for model in models:
        if model.project.name in projects:
            projects[model.project.name].append(model)
        else:
            projects[model.project.name] = [model]
    P = namedtuple('P', 'models min_score max_score')
    for project in projects:
        scores = [model.key_metric for model in models]
        projects[project] = P(sorted(projects[project], key=lambda model: -model.key_metric), min(scores), max(scores))

    context = {
        "snapshot": snapshot,
        "features": columns,
        "projects": projects,
    }
    return render(request, "snapshots/snapshot.html", context)
