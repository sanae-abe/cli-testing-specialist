#!/usr/bin/env bash
#
# i18n-loader.sh - Lightweight internationalization (i18n) loader for CLI Testing Specialist
#
# Provides:
#   - Language detection from $LANG or $CLI_TEST_LANG
#   - Message file loading (i18n/ja.sh, i18n/en.sh)
#   - msg() helper function for retrieving localized messages
#
# Security features:
#   - Whitelist-based language code validation
#   - i18n file format validation
#   - Read-only global variables
#   - Protection against code injection
#
# Usage:
#   source utils/i18n-loader.sh
#   load_i18n_once
#   echo "$(msg cli_analysis_started)"
#

# ============================================
# Global Variables
# ============================================

# Associative array for storing messages (global scope)
# Only declare if not already declared (prevents duplicate declaration errors)
if ! declare -p MESSAGES &>/dev/null; then
    declare -A MESSAGES
fi

# Flag to track whether i18n has been loaded (prevents duplicate loading)
I18N_LOADED=false

# ============================================
# Language Detection
# ============================================

# Detects the language code from environment variables
# Returns a whitelisted language code (ja|en), defaults to 'en'
#
# Environment variables (in order of precedence):
#   1. CLI_TEST_LANG (explicit override)
#   2. LANG (system locale)
#
# Returns:
#   Language code: 'ja' or 'en'
detect_language() {
    local lang="${LANG:-en_US.UTF-8}"
    local lang_code="${lang%%_*}"  # Extract language code (en_US.UTF-8 -> en)

    # Whitelist validation (security measure)
    case "$lang_code" in
        ja) echo "ja" ;;
        en) echo "en" ;;
        *)  echo "en" ;;  # Default to English for unknown locales
    esac
}

# ============================================
# i18n File Validation
# ============================================

# Validates an i18n file format before loading
# Ensures the file exists and contains the required structure
#
# Args:
#   $1 - Path to i18n file
#
# Returns:
#   0 if valid, 1 if invalid
validate_i18n_file() {
    local file="$1"

    # Check file existence
    if [[ ! -f "$file" ]]; then
        echo "❌ Error: i18n file not found: $file" >&2
        return 1
    fi

    # Validate file format (must contain MESSAGES assignments)
    if ! grep -q '^MESSAGES\[' "$file"; then
        echo "❌ Error: Invalid i18n file format: $file" >&2
        echo "💡 Expected format: MESSAGES[key]=\"value\"" >&2
        return 1
    fi

    return 0
}

# ============================================
# i18n Loading
# ============================================

# Loads the i18n file once based on detected language
# Uses lazy loading pattern - only loads when first called
#
# Environment variables:
#   CLI_TEST_LANG - Explicit language override (ja|en)
#   LANG - System locale (fallback)
#
# Returns:
#   0 on success, 1 on failure
load_i18n_once() {
    # Skip if already loaded (performance optimization)
    if [[ "$I18N_LOADED" == "true" ]]; then
        return 0
    fi

    # Detect language code (with whitelist validation)
    local lang_code="${CLI_TEST_LANG:-$(detect_language)}"

    # Re-validate whitelist (security hardening)
    case "$lang_code" in
        ja|en) ;;
        *)
            echo "❌ Error: Invalid language code: $lang_code" >&2
            echo "💡 Supported languages: ja, en" >&2
            lang_code="en"  # Fallback to English
            ;;
    esac

    # Construct i18n file path
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    local i18n_file="${script_dir}/i18n/${lang_code}.sh"

    # Validate file before loading
    validate_i18n_file "$i18n_file" || return 1

    # Load i18n file (safe source)
    # Note: Cannot unset MESSAGES if readonly, so check first
    if readonly -p 2>/dev/null | grep -q "^declare -[^=]*r[^=]* MESSAGES"; then
        # MESSAGES is already readonly, skip loading
        I18N_LOADED=true
        return 0
    fi

    # shellcheck source=/dev/null
    source "$i18n_file" || {
        echo "❌ Error: Failed to load i18n file: $i18n_file" >&2
        return 1
    }

    # Make variables read-only (security measure)
    readonly MESSAGES

    # Only set CLI_TEST_LANG as readonly if not already readonly
    if ! readonly -p 2>/dev/null | grep -q "^declare -[^=]*r[^=]* CLI_TEST_LANG="; then
        readonly CLI_TEST_LANG="$lang_code"
    fi
    I18N_LOADED=true

    return 0
}

# ============================================
# Message Retrieval
# ============================================

# Retrieves a localized message by key
# Returns the message or a fallback error message if key is missing
# Supports printf-style formatting with additional arguments
#
# Args:
#   $1 - Message key (e.g., 'cli_analysis_started')
#   $2... - Optional printf-style arguments for %s placeholders
#
# Returns:
#   Localized message string (formatted if arguments provided)
msg() {
    local key="$1"
    shift || true

    # Check if MESSAGES array is declared and has content
    if [[ ${#MESSAGES[@]} -eq 0 ]]; then
        echo "[i18n not loaded]"
        return 1
    fi

    # Check if key exists
    if [[ -z "${MESSAGES[$key]+x}" ]]; then
        echo "[Missing i18n key: $key]"
        return 1
    fi

    local message="${MESSAGES[$key]}"

    # If additional arguments provided, use printf formatting
    if [[ $# -gt 0 ]]; then
        # shellcheck disable=SC2059
        printf "$message" "$@"
    else
        echo "$message"
    fi
}

# ============================================
# Initialization
# ============================================

# Set default language (only if not already set)
if [[ -z "${CLI_TEST_LANG:-}" ]]; then
    CLI_TEST_LANG="$(detect_language)"
    export CLI_TEST_LANG
fi
