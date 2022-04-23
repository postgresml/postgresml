from django.urls import path

from . import views
from .views import root, projects, models, snapshots, deployments


urlpatterns = [
    path('', root.index, name='index'),
    path('projects/', projects.index, name='projects'),
    path('projects/<int:id>', projects.index, name='project'),
]
