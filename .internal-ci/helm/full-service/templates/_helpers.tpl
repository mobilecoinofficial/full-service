{{/* Copyright (c) 2018-2022 The MobileCoin Foundation */}}
{{/* Expand the name of the chart. */}}
{{- define "fullService.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "fullService.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/* Create chart name and version as used by the chart label. */}}
{{- define "fullService.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" | trimSuffix "." }}
{{- end }}

{{/* Common labels */}}
{{- define "fullService.labels" -}}
helm.sh/chart: {{ include "fullService.chart" . }}
{{ include "fullService.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | trunc 63 | trimSuffix "-" | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/* Selector labels */}}
{{- define "fullService.selectorLabels" -}}
app.kubernetes.io/name: {{ include "fullService.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/* fullService ConfigMap Name */}}
{{- define "fullService.configMap.name" -}}
  {{- if .Values.fullService.configMap.external }}
    {{- .Values.fullService.configMap.name }}
  {{- else }}
    {{- include "fullService.fullname" . }}
  {{- end }}
{{- end }}

{{- define "fullService.secret.name" -}}
  {{- if .Values.fullService.secret.external }}
    {{- .Values.fullService.secret.name }}
  {{- else }}
    {{- include "fullService.fullname" . }}
  {{- end }}
{{- end }}

{{/* backupSidecar Secret Name */}}
{{- define "fullService.backupsSidecar.secret.name" -}}
  {{- if .Values.backupsSidecar.secret.external }}
    {{- .Values.backupsSidecar.secret.name }}
  {{- else }}
    {{- include "fullService.fullname" . }}-{{ .Values.backupsSidecar.secret.name }}
  {{- end }}
{{- end }}

{{/* fullService image */}}
{{- define "fullService.image" -}}
{{ .Values.fullService.image.org | default .Values.image.org }}/{{ .Values.fullService.image.name }}:{{ .Values.fullService.image.tag | default .Chart.AppVersion }}
{{- end }}

{{/* backupsSidecar image */}}
{{- define "fullService.backupsSidecar.image" -}}
{{ .Values.backupsSidecar.image.org | default .Values.image.org }}/{{ .Values.backupsSidecar.image.name }}:{{ .Values.backupsSidecar.image.tag | default .Chart.AppVersion }}
{{- end }}
