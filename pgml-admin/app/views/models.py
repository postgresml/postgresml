from typing import OrderedDict
from collections import namedtuple
import json
import logging

from django.shortcuts import render, get_object_or_404
from django.views.generic import DetailView, ListView
from rest_framework import viewsets

from app.models import Model
from app.serializers import ModelSerializer


def default_context(context):
    """Specify values for the base template."""
    return {"topic": "models", **context}


class ModelView(DetailView):
    """View of a particular trained model."""

    model = Model
    template_name = "models/model.html"

    def get_context_data(self, **kwargs):
        context = default_context(super().get_context_data(**kwargs))
        object = context["object"]
        context["title"] = object.project.name + " - " + object.algorithm_name

        search_results = object.metrics.get("search_results", None)
        if search_results:
            graphs = {
                "test_score": ["Test Score", "mean_test_score", "std_test_score"],
                "fit_time": ["Fit Time", "mean_fit_time", "std_fit_time"], 
                "score_time": ["Score Time", "mean_score_time", "std_score_time"],
            }
            graph_data = {}
            for param, values in object.search_params.items():
                graph_data[param] = {"values": values}
                for graph, (title, means, stds) in graphs.items():
                    graph_data[param][graph] = {
                        "title": title,
                        "std": search_results[stds],
                        "y": json.dumps(search_results[means]),
                        "x": json.dumps(search_results["param_" + param]),
                    }
            context["graph_data"] = graph_data
        context["search_results"] = search_results
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


class ModelViewSet(viewsets.ModelViewSet):
    queryset = Model.objects.all().prefetch_related("project", "snapshot")
    serializer_class = ModelSerializer
