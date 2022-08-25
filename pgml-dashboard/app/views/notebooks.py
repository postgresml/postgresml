from django.shortcuts import render, get_object_or_404
from django.urls import reverse_lazy, reverse
from django.http import HttpResponse, HttpResponseRedirect
from django import forms
from django.db import transaction
from django.utils import timezone
from django.template.loader import render_to_string
from django.utils.html import strip_tags

from app.models import Notebook, NotebookCell
import time


def index(request):
    return render(
        request,
        "notebooks/index.html",
        {
            "notebooks": Notebook.objects.order_by("id").all(),
            "topic": "notebooks",
        },
    )


def notebook(request, pk):
    """Render a notebook."""
    notebook = get_object_or_404(Notebook, pk=pk)
    cells = notebook.notebookcell_set.all().order_by("cell_number").filter(deleted_at__isnull=True)

    return render(
        request,
        "notebooks/notebook.html",
        {
            "cells": cells,
            "notebook": notebook,
            "title": f"{notebook.name} - PostgresML",
            "topic": "notebooks",
            "bust_cache": time.time(),
        },
    )


class NotebookForm(forms.Form):
    name = forms.CharField()


def create_notebook(request):
    """Create a notebook."""
    notebook_form = NotebookForm(request.POST)

    if notebook_form.is_valid():
        notebook = Notebook.objects.create(
            name=notebook_form.cleaned_data["name"],
        )

        return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": notebook.pk}))
    else:
        return HttpResponse(status=400)


def rename_notebook(request, pk):
    """Rename the notebook."""
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
            "bust_cache": time.time(),
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
        ),
    )


def add_notebook_cell(request, pk):
    """Add a new notebook cell."""
    cell_form = NotebookCellForm(request.POST)

    if not cell_form.is_valid():
        print(cell_form.errors)
        return HttpResponse(cell_form.errors, status=400)

    # Prevent concurrent updates & data races.
    with transaction.atomic():
        notebook = Notebook.objects.select_for_update().get(pk=pk)
        last_cell = (
            NotebookCell.objects.filter(notebook=notebook, deleted_at__isnull=True).order_by("cell_number").last()
        )

        if cell_form.cleaned_data["cell_type"] == str(NotebookCell.SQL):
            cell_type = NotebookCell.SQL
        else:
            cell_type = NotebookCell.MARKDOWN

        cell = NotebookCell.objects.create(
            notebook=notebook,
            contents=strip_tags(cell_form.cleaned_data["contents"].strip()),
            cell_number=(last_cell.cell_number + 1 if last_cell else 1),
            cell_type=cell_type,
        )

    # Render outside the transaction because it cause it to rollback
    # if there is an error in the cell SQL.
    cell.render()

    return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": notebook.pk}))


def edit_notebook_cell(request, notebook_pk, cell_pk):
    """Edit a notebook cell."""
    notebook = get_object_or_404(Notebook, pk=notebook_pk)
    cell = get_object_or_404(NotebookCell, pk=cell_pk)

    # Start editing a cell.
    if request.method == "GET":
        return render(
            request,
            "notebooks/cell.html",
            {
                "cell": cell,
                "notebook": notebook,
                "edit": True,
                "bust_cache": time.time(),  # Turbo won't submit a get form if it already did.
                "selected": True,
            },
        )

    # Submit cell edit.
    if request.method == "POST":
        cell_form = NotebookCellForm(request.POST)

        if not cell_form.is_valid():
            return HttpResponse(request, status=400)

        if cell_form.cleaned_data["cell_type"] == str(NotebookCell.SQL):
            cell_type = NotebookCell.SQL
        else:
            cell_type = NotebookCell.MARKDOWN

        cell.contents = strip_tags(cell_form.cleaned_data["contents"].strip())
        cell.cell_type = cell_type

        # If cell was changed to Markdown, remove execution time.
        if not cell.code:
            cell.execution_time = None

        cell.save()
        cell.render()

        return render(
            request,
            "notebooks/cell.html",
            {
                "cell": cell,
                "notebook": notebook,
                "bust_cache": time.time(),
            },
        )


@transaction.atomic
def remove_notebook_cell(request, notebook_pk, cell_pk):
    """Delete a notebook cell."""
    cell = get_object_or_404(NotebookCell, pk=cell_pk, notebook__pk=notebook_pk)

    # Actually delete the cell.
    if request.POST.get("confirm"):
        cell.deleted_at = timezone.now()
        cell.save()

        # Re-order cells after a delete.
        notebook = get_object_or_404(Notebook, pk=notebook_pk)
        cells = notebook.notebookcell_set.order_by("cell_number").filter(deleted_at__isnull=True)
        cell_number = 1
        for c in cells:
            c.cell_number = cell_number
            c.save()

            cell_number += 1

        return render(
            request,
            "notebooks/cell.html",
            {
                "cell": cell,
                "notebook": cell.notebook,
                "bust_cache": time.time(),
            },
        )

    # Leave it alone.
    else:
        return render(
            request,
            "notebooks/undo.html",
            {
                "cell": cell,
                "notebook": cell.notebook,
                "bust_cache": time.time(),
            },
        )


def reset_notebook(request, pk):
    """Remove renderings from all cells that can be executed, e.g. SQL."""
    notebook = get_object_or_404(Notebook, pk=pk)
    notebook.reset()

    return HttpResponseRedirect(reverse_lazy("notebooks/notebook", kwargs={"pk": pk}))


def play_notebook_cell(request, notebook_pk, cell_pk):
    """Execute/render the notebook cell."""
    cell = get_object_or_404(NotebookCell, pk=cell_pk)
    cell.render()

    return render(
        request,
        "notebooks/cell.html",
        {
            "cell": cell,
            "notebook": cell.notebook,
            "bust_cache": time.time(),
            "selected": True,
        },
    )
