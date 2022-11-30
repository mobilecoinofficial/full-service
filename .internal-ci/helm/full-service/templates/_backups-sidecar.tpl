{{/* Copyright (c) 2018-2022 The MobileCoin Foundation */}}
{{- define "fullService.backupsSidecarContainer" }}
- name: backups-sidecar
  securityContext:
    capabilities:
      drop:
      - all
    readOnlyRootFilesystem: true
  image: {{ include "fullService.backupsSidecar.image" . }}
  imagePullPolicy: Always
  args:
  - "while true; do sleep 30; /app/bin/full-service/backup.sh; sleep 3600; done"
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
    readOnly: true
  - name: tmp
    mountPath: /tmp
{{- end }}
