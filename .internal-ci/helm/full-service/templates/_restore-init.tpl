{{/* Copyright (c) 2018-2022 The MobileCoin Foundation */}}
{{- define "fullService.restoreInitContainer" }}
- name: restore-sidecar
  securityContext:
    capabilities:
      drop:
      - all
    readOnlyRootFilesystem: true
  image: {{ include "fullService.backupsSidecar.image" . }}
  imagePullPolicy: Always
  args:
  - /app/bin/full-service/restore.sh
  envFrom:
  - secretRef:
      name: {{ include "fullService.backupsSidecar.secret.name" . }}
  env:
  - name: NAMESPACE
    valueFrom:
      fieldRef:
        fieldPath: metadata.namespace
  volumeMounts:
  - name: data
    mountPath: /data
  - name: tmp
    mountPath: /tmp
{{- end }}
