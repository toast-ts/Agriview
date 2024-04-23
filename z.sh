#!/bin/bash

clear && docker build -t git.toast-server.net/toast/agriview:latest . && docker push git.toast-server.net/toast/agriview
