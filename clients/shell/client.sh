#!/bin/sh
health_path() { printf '%s/v1/health' "${1%/}"; }
