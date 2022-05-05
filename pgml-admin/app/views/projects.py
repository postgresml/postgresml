from typing import OrderedDict
from collections import namedtuple

from django.shortcuts import render, get_object_or_404
from django.views.generic import DetailView, ListView
from django.views.generic.base import TemplateView
from rest_framework import viewsets

from app.models import Project, InformationSchemaTable
from app.serializers import ProjectSerializer


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
                min(0, max([0, min(scores)])),
                max(scores),
                project.id,
            )

        return {
            **context,
            "projects": projects,
        }


class NewProjectView(TemplateView):
    template_name = "projects/new.html"

    def get_context_data(self, **kwargs):
        context = default_context(super().get_context_data(**kwargs))
        context["tables"] = InformationSchemaTable.objects.filter(table_schema="pgml", table_name__in=["diabetes", 'digits', 'iris', 'linnerud', 'wine', 'breast_cancer', 'california_housing'])
        context["controller"] = "new-project"
        return context


class ProjectViewSet(viewsets.ModelViewSet):
    queryset = Project.objects.all()
    serializer_class = ProjectSerializer

