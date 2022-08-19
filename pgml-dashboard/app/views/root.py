from django.shortcuts import render, reverse

from django.http import HttpResponse, HttpResponseRedirect
from django.utils import timezone
from rest_framework.decorators import api_view, permission_classes, action
from rest_framework.response import Response
from rest_framework import viewsets

from app import models
from app.serializers import RequestSerializer
from request.models import Request

from datetime import timedelta


def index(request):
    """Root of the dashboard."""
    return HttpResponseRedirect(reverse("notebooks"))


def set_auth_cookie(request):
    """Persist the user auth token in the cookie."""
    token = request.GET.get("dashboard_auth")

    if not token:
        return HttpResponse(status=400)

    response = HttpResponseRedirect(reverse("index"))
    response.set_cookie("dashboard_auth", token, secure=True, httponly=True)

    return response


class RequestViewSet(viewsets.ModelViewSet):
    """See dashboard activity.

    Helps us to determine if the dashboard is still used in our free cloud
    or if we can shut it down to save on costs.

    When JWT auth is enabled, this view is protected against unauthorized access.
    """

    queryset = Request.objects.all().order_by("-pk")
    serializer_class = RequestSerializer

    @action(detail=False)
    def purge(self, request):
        """Delete everything but the last `hours` hours of requests. Defaults to 24 hours."""
        hours = int(request.GET.get("hours", 24))
        self.queryset.filter(time__lte=timezone.now() - timedelta(hours=hours)).delete()
        return Response(data={})
