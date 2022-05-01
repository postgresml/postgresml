from typing import OrderedDict
from collections import namedtuple
import json
import logging

from django.shortcuts import render, get_object_or_404
from django.views.generic import DetailView, ListView

from app.models import Model



def default_context(context):
    """Specify values for the base template."""
    return {"topic": "Models", **context}


class ModelView(DetailView):
    """View of a particular trained model."""

    model = Model
    template_name = "models/model.html"

    def get_context_data(self, **kwargs):
        context = default_context(super().get_context_data(**kwargs))
        # context["object"].metrics = json.dumps(context["object"].metrics, indent=2)
        context["title"] = context["object"].project.name + " - " + context["object"].algorithm_name
        return context


class ModelListView(ListView):
    """List of trained models for all project."""

    model = Model
    template_name = "models/index.html"

    def get_context_data(self, **kwargs):
        models = Model.objects.order_by("created_at").all().prefetch_related("project")
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

        context = default_context({"projects": projects})
        return context
