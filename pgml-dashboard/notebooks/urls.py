from django.urls import path

from notebooks.views import notebook, add_notebook_line, remove_notebook_line

urlpatterns = [
    path("notebook/<int:pk>/", notebook, name="notebooks/notebook"),
    path("notebook/<int:pk>/line/add/", add_notebook_line, name="notebooks/line/add"),
    path("notebook/<int:notebook_pk>/line/<int:line_pk>/remove/", remove_notebook_line, name="notebooks/line/remove"),
]
