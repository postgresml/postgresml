from django.contrib import admin
from notebooks.models import *

admin.site.register(Notebook)
admin.site.register(NotebookCell)
