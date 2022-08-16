from django.shortcuts import render, get_object_or_404
from django.urls import reverse_lazy, reverse
from django.http import HttpResponse, HttpResponseRedirect
from django import forms
from django.db import transaction
from django.utils import timezone

from notebooks.models import *


def notebook(request, pk):
    """Render a notebook."""
    notebook = get_object_or_404(Notebook, pk=pk)

    return render(
        request,
        "notebooks/notebook.html",
        {
            "cells": notebook.notebookcell_set.all().filter(deleted_at__isnull=True).order_by("cell_number"),
            "notebook": notebook,
        },
    )


class NotebookForm(forms.Form):
    name = forms.CharField()


def create_notebook(request):
    notebook_form = NotebookForm(request.POST)

    if notebook_form.is_valid():
        notebook = Notebook.objects.create(
            name=notebook_form.cleaned_data["name"],
        )

        return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": notebook.pk}))
    else:
        return HttpResponse(status=400)


def rename_notebook(request, pk):
    notebook_form = NotebookForm(request.POST)

    if notebook_form.is_valid():
        Notebook.objects.filter(pk=pk).update(name=notebook_form.cleaned_data["name"])
        return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": pk}))
    else:
        return HttpResponse(status=400)


def notebook_cell(request, notebook_pk, cell_pk):
    """Render a single notebook cell."""
    notebook = get_object_or_404(Notebook, pk=notebook_pk)
    cell = get_object_or_404(NotebookCell, pk=cell_pk)

    return render(
        request,
        "notebooks/cell.html",
        {
            "cell": cell,
            "notebook": cell.notebook,
        },
    )


class NotebookCellForm(forms.Form):
    contents = forms.CharField(required=False)
    cell_type = forms.ChoiceField(
        required=True,         
        choices=(
            (
                NotebookCell.MARKDOWN,
                "Markdown",
            ),
            (
                NotebookCell.SQL,
                "SQL",
            ),
        ))


@transaction.atomic
def add_notebook_cell(request, pk):
    """Add a new notebook cell."""
    notebook = Notebook.objects.select_for_update().get(pk=pk)
    cell_form = NotebookCellForm(request.POST)
    last_cell = NotebookCell.objects.filter(notebook=notebook, deleted_at__isnull=True).order_by("cell_number").last()

    if cell_form.is_valid():
        contents = cell_form.cleaned_data["contents"].strip()

        if cell_form.cleaned_data["cell_type"] == str(NotebookCell.SQL):
            cell_type = NotebookCell.SQL
        else:
            cell_type = NotebookCell.MARKDOWN

        cell = NotebookCell.objects.create(
            notebook=notebook,
            contents=contents,
            cell_number=(last_cell.cell_number + 1 if last_cell else 1),
            cell_type=cell_type,
        )

        return HttpResponseRedirect(
            reverse_lazy("notebooks/cell/get", kwargs={"notebook_pk": notebook.pk, "cell_pk": cell.pk})
        )
    else:
        print(cell_form.errors)
        return HttpResponse(cell_form.errors, status=400)


def edit_notebook_cell(request, notebook_pk, cell_pk):
    notebook = get_object_or_404(Notebook, pk=notebook_pk)
    old_cell = get_object_or_404(NotebookCell, pk=cell_pk)
    cell_form = NotebookCellForm(request.POST)

    if cell_form.is_valid():
        contents = cell_form.cleaned_data["contents"].strip()

        if cell_form.cleaned_data["cell_type"] == str(NotebookCell.SQL):
            cell_type = NotebookCell.SQL
        else:
            cell_type = NotebookCell.MARKDOWN

        with transaction.atomic():
            new_cell = NotebookCell.objects.create(
                notebook=notebook,
                contents=contents,
                version=old_cell.version + 1,
                cell_number=old_cell.cell_number,
                cell_type=cell_type,
            )
            old_cell.delete()
        return render(
            request,
            "notebooks/cell.html",
            {
                "cell": new_cell,
                "notebook": notebook,
            },
        )
    else:
        return HttpResponse(request, status=400)


@transaction.atomic
def remove_notebook_cell(request, notebook_pk, cell_pk):
    cell = get_object_or_404(NotebookCell, pk=cell_pk, notebook__pk=notebook_pk)
    cell.deleted_at = timezone.now()
    cell.save()

    return render(
        request,
        "notebooks/undo.html",
        {
            "cell": cell,
            "notebook": cell.notebook,
        },
    )


def undo_remove_notebook_cell(request, notebook_pk, cell_pk):
    cell = get_object_or_404(NotebookCell, pk=cell_pk, notebook__pk=notebook_pk)
    cell.deleted_at = None
    cell.save()

    return render(
        request,
        "notebooks/cell.html",
        {
            "cell": cell,
            "notebook": cell.notebook,
        },
    )
