from typing import OrderedDict
from collections import namedtuple

from django.shortcuts import render, get_object_or_404
from django.views.generic import DetailView, ListView

from app.models import Project



def default_context(context):
    return {"topic": "Projects", **context}


def index(request):
    projects = Project.objects.all()
    context = default_context({"title": "Projects", "projects": projects})
    return render(request, "projects/index.html", context)


def project(request, id):
    if request.method == "GET":
        return get(request, id)


class ProjectView(DetailView):
    model = Project
    template_name = "projects/project.html"

    def get_context_data(self, **kwargs):
        context = default_context(super().get_context_data(**kwargs))

        models = context["object"].models().order_by("created_at").all().prefetch_related("project")
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
                max([0, min(scores)]),
                max(scores),
                project.id,
            )

        return {
            **context,
            "projects": projects,
        }
