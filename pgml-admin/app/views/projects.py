from typing import OrderedDict
from collections import namedtuple

from django.shortcuts import render, get_object_or_404
from django.views.generic import DetailView, ListView
from django.views.generic.base import TemplateView
from django.db import connection

# DRF
from rest_framework import viewsets
from rest_framework.response import Response
from rest_framework import status
from rest_framework.decorators import action

from app.models import Project
from app.serializers import ProjectSerializer


def default_context(context):
    return {"topic": "projects", **context}


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


class ProjectViewSet(viewsets.ModelViewSet):
    queryset = Project.objects.all()
    serializer_class = ProjectSerializer


class NewProjectView(TemplateView):
    template_name = "projects/new.html"


class TableView(viewsets.ViewSet):
    """View handling table/view metadata."""
    permission_classes = []

    @staticmethod
    def _get_table(table_name):

        if "." in table_name:
            schema_name, table_name = tuple(table_name.split("."))
        else:
            schema_name, table_name = "public", table_name

        with connection.cursor() as cursor:
            cursor.execute("""
                SELECT table_schema, table_name
                FROM information_schema.tables
                WHERE table_schema = %s
                AND table_name = %s
            """, [schema_name, table_name])

            result = cursor.fetchone()
        return result[0], result[1]

    def list(self, request):
        if "table_name" not in request.GET:
            return Response(status=status.HTTP_400_BAD_REQUEST)

        table_name = request.GET["table_name"]

        if "." in table_name:
            schema_name, table_name = tuple(table_name.split("."))
        else:
            schema_name, table_name = "public", table_name

        with connection.cursor() as cursor:
            cursor.execute("""
                SELECT table_schema, table_name
                FROM information_schema.tables
                WHERE table_schema = %s
                AND table_name = %s
            """, [schema_name, table_name])

            result = cursor.fetchone()

        if result:
            return Response(data={
                "table_name": table_name,
                "table_schema": schema_name,
            })
        else:
            return Response(status=status.HTTP_404_NOT_FOUND)

    @action(detail=False)
    def sample(self, request):
        if "table_name" not in request.GET:
            return Response(status=status.HTTP_400_BAD_REQUEST)

        table_name = request.GET["table_name"]

        if "." in table_name:
            schema_name, table_name = tuple(table_name.split("."))
        else:
            schema_name, table_name = "public", table_name

        with connection.cursor() as cursor:
            cursor.execute("""
                SELECT table_schema, table_name
                FROM information_schema.tables
                WHERE table_schema = %s
                AND table_name = %s
            """, [schema_name, table_name])

            result = cursor.fetchone()

        if not result:
            return Response(status=status.HTTP_404_NOT_FOUND)

        # No SQL injections
        schema_name, table_name = result[0], result[1]

        with connection.cursor() as cursor:
            cursor.execute(f"""
                SELECT * FROM
                {schema_name}.{table_name}
                LIMIT 10
            """)

            result = cursor.fetchall()

            return render(request, "projects/sample.html", {
                "columns": [desc[0] for desc in cursor.description],
                "rows": result,
            })

    @action(detail=False)
    def columns(self, request):
        if "table_name" not in request.GET:
            return Response(status=status.HTTP_400_BAD_REQUEST)

        schema_name, table_name = TableView._get_table(request.GET["table_name"])

        with connection.cursor() as cursor:
            cursor.execute(f"""
                SELECT * FROM
                {schema_name}.{table_name}
                LIMIT 1
            """)

            result = cursor.fetchone()
            names = [desc[0] for desc in cursor.description]

            return render(request, "projects/target.html", {
                "columns": [
                    {
                        "name": names[i],
                        "data_type": type(result[i]).__name__
                    } for i in range(len(result))
                ]
            })
