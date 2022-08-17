from django.urls import path
from explorer.views import *

urlpatterns = [
    path("query/<int:pk>/", query, name="explorer/query"),
    path("query/create/", create_query, name="explorer/query/create"),
    path("", explorer, name="explorer/explorer"),
]
