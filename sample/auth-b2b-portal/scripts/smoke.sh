#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
COOKIE_JAR="${COOKIE_JAR:-/tmp/dolang-auth-b2b.cookie}"

echo "[1] login"
LOGIN_JSON="$(curl -sS \
  -c "${COOKIE_JAR}" \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"password123"}' \
  "${BASE_URL}/auth/login")"
echo "${LOGIN_JSON}"

echo "[2] me via cookie"
curl -sS \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/me"
echo

echo "[3] me via bearer"
ACCESS_TOKEN="$(printf '%s' "${LOGIN_JSON}" | jq -r '.access_token')"
curl -sS \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  "${BASE_URL}/me"
echo

echo "[4] switch workspace via cookie + csrf"
CSRF_TOKEN="$(printf '%s' "${LOGIN_JSON}" | jq -r '.csrf_token')"
curl -sS \
  -X POST \
  -b "${COOKIE_JAR}" \
  -H 'Content-Type: application/json' \
  -H "X-CSRF-Token: ${CSRF_TOKEN}" \
  -d '{"workspace_id":"team_red"}' \
  "${BASE_URL}/workspaces/current/switch"
echo

echo "[5] refresh pair"
REFRESH_TOKEN="$(printf '%s' "${LOGIN_JSON}" | jq -r '.refresh_token')"
curl -sS \
  -X POST \
  -H 'Content-Type: application/json' \
  -d "{\"refresh_token\":\"${REFRESH_TOKEN}\"}" \
  "${BASE_URL}/auth/refresh"
echo
