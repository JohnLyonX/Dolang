#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
COOKIE_JAR="${COOKIE_JAR:-/tmp/dolang-auth-b2b.cookie}"
EDITOR_COOKIE_JAR="${EDITOR_COOKIE_JAR:-/tmp/dolang-auth-b2b-editor.cookie}"

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

echo "[6] editor login"
EDITOR_LOGIN_JSON="$(curl -sS \
  -c "${EDITOR_COOKIE_JAR}" \
  -H 'Content-Type: application/json' \
  -d '{"username":"editor","password":"password123"}' \
  "${BASE_URL}/auth/login")"
echo "${EDITOR_LOGIN_JSON}"
EDITOR_ACCESS_TOKEN="$(printf '%s' "${EDITOR_LOGIN_JSON}" | jq -r '.access_token')"
EDITOR_CSRF_TOKEN="$(printf '%s' "${EDITOR_LOGIN_JSON}" | jq -r '.csrf_token')"

echo "[7] editor create draft via cookie + csrf"
NEWS_SLUG="editor-news-$(date +%s)"
CREATE_JSON="$(curl -sS \
  -X POST \
  -b "${EDITOR_COOKIE_JAR}" \
  -H 'Content-Type: application/json' \
  -H "X-CSRF-Token: ${EDITOR_CSRF_TOKEN}" \
  -d "{\"slug\":\"${NEWS_SLUG}\",\"title\":\"Editor Draft\",\"summary\":\"Created from smoke\",\"body\":\"Smoke test article body\",\"category_id\":\"cat_product\"}" \
  "${BASE_URL}/api/admin/news")"
echo "${CREATE_JSON}"
ARTICLE_ID="$(printf '%s' "${CREATE_JSON}" | jq -r '.article_id')"

echo "[8] editor publish draft via bearer should be forbidden"
curl -sS \
  -X POST \
  -H "Authorization: Bearer ${EDITOR_ACCESS_TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{}' \
  "${BASE_URL}/api/admin/news/${ARTICLE_ID}/publish"
echo

echo "[9] admin publish draft via bearer"
curl -sS \
  -X POST \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H 'Content-Type: application/json' \
  -d '{}' \
  "${BASE_URL}/api/admin/news/${ARTICLE_ID}/publish"
echo

echo "[10] public news list includes published article"
curl -sS \
  "${BASE_URL}/api/news"
echo
