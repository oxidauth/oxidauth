{{/* vim: set filetype=mustache: */}}

{{/*
    Expand the name of the chart.
*/}}
{{- define "oxidauth.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
    Create a default fully qualified app name.
    We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
    If release name contains chart name it will be used as a full name.
*/}}
{{- define "oxidauth.fullname" -}}
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

{{/*
    Image Pull Secrets
*/}}
{{- define "oxidauth.imagePullSecret" }}
{{- printf "{\"auths\": {\"%s\": {\"auth\": \"%s\"}}}" .Values.registryCredentials.registry (printf "%s:%s" .Values.registryCredentials.username .Values.registryCredentials.password | b64enc) | b64enc }}
{{- end }}

{{/*
    Create chart name and version as used by the chart label.
*/}}
{{- define "oxidauth.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
    Common labels
*/}}
{{- define "oxidauth.labels" -}}
helm.sh/chart: {{ include "oxidauth.chart" . }}
{{ include "oxidauth.selectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: {{ include "oxidauth.fullname" . }}
{{- end }}

{{/*
    Common Selector labels
*/}}
{{- define "oxidauth.selectorLabels" -}}
app.kubernetes.io/name: {{ include "oxidauth.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
    Env keys whose values are secret material: rendered into the api Secret
    (secret.yaml) and keyRef'd via secretKeyRef (deployment.yaml) instead of
    the ConfigMap. Overridable wholesale with values.envSecretKeys. Renders a
    JSON OBJECT ({"KEY":true,…}) for callers to `fromJson` + `hasKey`: helm
    4's fromJson only decodes JSON objects and its membership helpers broke
    the sprig list form, so dict+hasKey is the portable shape.
*/}}
{{- define "oxidauth.secretEnvKeys" -}}
{{- $default := list "DATABASE_URL" "READ_DATABASE_URL" "OXIDAUTH_USERNAME_PASSWORD_PEPPER" -}}
{{- $keys := dict -}}
{{- range $key := default $default .Values.envSecretKeys }}{{ $_ := set $keys $key true }}{{- end -}}
{{- toJson $keys -}}
{{- end -}}

{{/*
    API Shorthand for component names
*/}}
{{- define "oxidauth.api.name" -}}
{{- include "oxidauth.fullname" . -}}-api
{{- end -}}

{{/*
    App API Labels
*/}}
{{- define "oxidauth.api.labels" -}}
{{ include "oxidauth.labels" . }}
app.kubernetes.io/component: api
app.kubernetes.io/version: {{ $.Values.api.image.tag }}
{{- end }}

{{/*
    App API Selectors
*/}}
{{- define "oxidauth.api.selectorLabels" -}}
{{ include "oxidauth.selectorLabels" . }}
app.kubernetes.io/component: api
{{- end }}

{{/*
    App Web Shorthand for component names
*/}}
{{- define "oxidauth.web.name" -}}
{{- include "oxidauth.fullname" . -}}-web
{{- end -}}

{{/*
    App Web Labels
*/}}
{{- define "oxidauth.web.labels" -}}
{{ include "oxidauth.labels" . }}
app.kubernetes.io/component: web
app.kubernetes.io/version: {{ $.Values.web.image.tag }}
{{- end }}

{{/*
    App Web Selectors
*/}}
{{- define "oxidauth.web.selectorLabels" -}}
{{ include "oxidauth.selectorLabels" . }}
app.kubernetes.io/component: web
{{- end }}