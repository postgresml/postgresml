from django.db import connection
from django.shortcuts import render, get_object_or_404
from django.utils.safestring import SafeString
from django.http import HttpResponse, HttpResponseRedirect
from django.urls import reverse_lazy
from django import forms

from app.models import UploadedData

import csv
import json
import codecs


class UploadForm(forms.Form):
    file = forms.FileField()
    has_header = forms.BooleanField(required=False)


def index(request):
    if request.method == "POST":
        form = UploadForm(request.POST, request.FILES)
        if not form.is_valid():
            return HttpResponse(status=400)

        file = request.FILES.get("file")
        if file.content_type not in ["text/csv", "application/json"]:
            return HttpResponse(status=400)
        else:
            try:
                upload = UploadedData.objects.create(
                    file_type=1 if file.content_type == "text/csv" else 2,
                )

                upload.create_table(file, form.cleaned_data.get("has_header", False))
            except Exception as e:
                return render(
                    request,
                    "uploader/index.html",
                    {
                        "error": str(e),
                        "topic": "uploader",
                    },
                    status=400,
                )
            return HttpResponseRedirect(reverse_lazy("uploader/uploaded", kwargs={"pk": upload.pk}))
    else:
        return render(
            request,
            "uploader/index.html",
            {
                "topic": "uploader",
            },
        )


def uploaded(request, pk):
    upload = UploadedData.objects.get(pk=pk)
    with connection.cursor() as cursor:
        cursor.execute(f"SELECT * FROM data_{upload.pk} LIMIT 11")
        columns = [col[0] for col in cursor.description]
        rows = cursor.fetchall()
    return render(
        request,
        "uploader/uploaded.html",
        {
            "columns": columns,
            "rows": rows[:10],
            "table_name": f"data_{upload.pk}",
            "redacted": len(rows) > 10,
            "topic": "uploader",
        },
    )
