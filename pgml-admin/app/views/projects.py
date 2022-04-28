from django.shortcuts import render, get_object_or_404

from app.models import Project

from django.views.generic import DetailView, ListView


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
        metrics = {model.algorithm_name: model.metrics for model in context["object"].models()}
        return {
            **context,
            "metrics": metrics,
        }
