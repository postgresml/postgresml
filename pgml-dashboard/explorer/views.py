from django.shortcuts import render, get_object_or_404
from django.http import HttpResponse, HttpResponseRedirect
from django.template.loader import render_to_string
from django import forms

from explorer.models import *


def query(request, pk):
    """Fetch a query rendering."""
    query = get_object_or_404(Query, pk=pk)
    return HttpResponse(query.html())


class QueryForm(forms.Form):
    query = forms.CharField()


def create_query(request):
    """Create a query, but don't execute it yet."""
    query_form = QueryForm(request.POST)

    if query_form.is_valid():
        query = Query.objects.create(
            contents=query_form.cleaned_data["query"],
        )

        return HttpResponseRedirect(reverse_lazy("explorer/query", kwargs={"pk": query.pk}))
    else:
        return HttpResponse(status=400)


def explorer(request):
    schemas = []

    with connection.cursor() as cursor:
        cursor.execute(
            """
            SELECT table_schema, COUNT(*) FROM information_schema.tables
            WHERE table_schema NOT IN ('information_schema', 'pg_catalog')
            GROUP BY 1
            ORDER BY 1 ASC
        """
        )
        rows = cursor.fetchall()

        for schema in rows:
            cursor.execute(
                """
                SELECT table_name FROM information_schema.tables
                WHERE table_schema = %s
            """,
                [
                    schema[0],
                ],
            )
            tables = cursor.fetchall()
            tables = list(map(lambda x: Table(x[0]), tables))
            schema = Schema(name=schema[0], tables=tables)
            schemas.append(schema)

    return render(
        request,
        "explorer/explorer.html",
        {
            "schemas": schemas,
        },
    )
