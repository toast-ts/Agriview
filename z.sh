#!/bin/bash

clear && docker build -t git.toast-server.net/toast/field-ownership-viewer:latest . && docker push git.toast-server.net/toast/field-ownership-viewer
