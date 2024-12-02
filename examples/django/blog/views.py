from django.http import HttpResponse

def home(request):
    return HttpResponse("Heelo, world. You're at the blog index.")
