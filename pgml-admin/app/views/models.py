from django.shortcuts import render, get_object_or_404
from django.views.generic import DetailView, ListView

from app.models import Model

import json


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
        context = default_context(super().get_context_data(**kwargs))
        return context
