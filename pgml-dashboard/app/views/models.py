from typing import OrderedDict
from collections import namedtuple
import json
import logging
import pickle
import io

from django.shortcuts import render, get_object_or_404
from django.views.generic import DetailView, ListView
from django.utils.safestring import SafeString
from django.db import connection

from rest_framework import viewsets
from rest_framework.decorators import action
from rest_framework.response import Response

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
        context["features"] = {
            feature: object.snapshot.analysis[f"{feature}_p50"]
            for feature in object.snapshot.columns.keys() - object.snapshot.y_column_name
        }

        if object.search:
            context["search_results"] = {}
            for key, value in object.metrics["search_results"].items():
                context["search_results"][key] = SafeString(json.dumps(value))
            context["best_index"] = object.metrics["search_results"]["best_index"]
            context["search_params"] = object.search_params
            context["search_graphs"] = {
                "test_score": "Test Score",
                "fit_time": "Fit Time",
                "score_time": "Score Time",
            }

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

    @action(detail=True, permission_classes=[], methods=["POST"])
    def predict(self, request, pk=None):
        model = get_object_or_404(Model, pk=pk)

        with connection.cursor() as cursor:
            cursor.execute(
                """
                SELECT pgml.model_predict(
                    model_id => %s,
                    features => %s
                )
            """,
                [model.pk, request.data],
            )
            result = cursor.fetchone()

            data = {
                "prediction": result[0],
            }

            return Response(data)
