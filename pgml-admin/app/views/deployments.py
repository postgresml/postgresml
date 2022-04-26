from django.shortcuts import render, get_object_or_404

from .. import models


def default_context(context):
    return {"topic": "Deployments", **context}


def index(request):
    deployments = models.Deployment.objects.all()
    context = default_context({"title": "Deployments", "deployments": deployments})
    return render(request, "deployments/index.html", context)


def deployment(request, id):
    if request.method == "GET":
        return get(request, id)


def get(request, id):
    deployment = get_object_or_404(models.Deployment, id=id)
    context = default_context(
        {"title": deployment.project.name + " - " + deployment.model.algorithm_name, "deployment": deployment}
    )
    return render(request, "deployments/deployment.html", context)
