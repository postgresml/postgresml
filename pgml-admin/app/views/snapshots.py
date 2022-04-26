from django.shortcuts import render, get_object_or_404

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
    context = default_context({"title": snapshot.relation_name, "snapshot": snapshot})
    return render(request, "snapshots/snapshot.html", context)
