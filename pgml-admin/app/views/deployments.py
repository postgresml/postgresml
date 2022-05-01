from typing import OrderedDict
from django.shortcuts import render, get_object_or_404

from .. import models


def default_context(context):
    return {"topic": "Deployments", **context}


def index(request):
    project_deployments = OrderedDict()
    for project in models.Project.objects.all():
        project_deployments[project] = project.deployment_set.order_by("-created_at").all()

    context = default_context({"title": "Deployments", "project_deployments": project_deployments})
    return render(request, "deployments/index.html", context)


def deployment(request, id):
    if request.method == "GET":
        return get(request, id)


def get(request, id):
    deployment = get_object_or_404(models.Deployment, id=id)
    context = default_context(
        {
            "title": deployment.project.name + " - " + deployment.model.algorithm_name,
            "deployment": deployment,
            "live": deployment.id == deployment.project.current_deployment.id,
        }
    )
    return render(request, "deployments/deployment.html", context)
