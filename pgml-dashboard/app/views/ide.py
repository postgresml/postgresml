from django.views.generic.base import TemplateView
from rest_framework.response import Response
from django.http import HttpResponse
from django.db import connection
from rest_framework.decorators import api_view, permission_classes
from django.shortcuts import render

from notebooks.models import Notebook

SPECIAL_QUERIES = {
    r"\d+": """
        SELECT n.nspname as "Schema",
      c.relname as "Name",
      CASE c.relkind WHEN 'r' THEN 'table' WHEN 'v' THEN 'view' WHEN 'm' THEN 'materialized view' WHEN 'i' THEN 'index' WHEN 'S' THEN 'sequence' WHEN 's' THEN 'special' WHEN 'f' THEN 'foreign table' WHEN 'p' THEN 'partitioned table' WHEN 'I' THEN 'partitioned index' END as "Type",
      pg_catalog.pg_get_userbyid(c.relowner) as "Owner",
      pg_catalog.pg_size_pretty(pg_catalog.pg_table_size(c.oid)) as "Size",
      pg_catalog.obj_description(c.oid, 'pg_class') as "Description"
        FROM pg_catalog.pg_class c
             LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind IN ('r','p','v','m','S','f','')
              AND n.nspname <> 'pg_catalog'
              AND n.nspname <> 'information_schema'
              AND n.nspname !~ '^pg_toast'
          AND pg_catalog.pg_table_is_visible(c.oid)
        ORDER BY 1,2;
    """,
}


class IdeView(TemplateView):
    template_name = "ide/index.html"

    def get_context_data(self, **kwargs):
        data = super().get_context_data(**kwargs)
        data["topic"] = "ide"
        data["notebooks"] = Notebook.objects.all()
        return data


@api_view(["POST"])
@permission_classes([])
def run_sql(request):
    query = request.data["query"].strip()

    if query in SPECIAL_QUERIES:
        query = SPECIAL_QUERIES[query]

    with connection.cursor() as cursor:
        try:
            cursor.execute("SET statement_timeout = '30s'")
            cursor.execute(query)
            results = cursor.fetchall()

            return render(
                request,
                "projects/sample.html",
                {
                    "columns": [desc[0] for desc in cursor.description],
                    "rows": results,
                },
            )
        except Exception as e:
            return HttpResponse(
                f"""
                <code>
<pre>
{e}
                    </pre>
                </code>
            """
            )
