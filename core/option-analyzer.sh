#!/usr/bin/env bash
#
# option-analyzer.sh - Option Type Inference Engine
# CLI Testing Specialist Agent v2.5.0
#
# Features:
# - YAML-driven option type inference
# - Numeric constraint extraction
# - Enumeration value extraction
# - Data-driven design for maintainability
#

set -euo pipefail
IFS=$'\n\t'

# Error trap
trap 'log_error_with_trace "Error at line $LINENO in option-analyzer.sh"' ERR

# Script directory detection（読み取り専用）
# SCRIPT_DIRが未定義の場合のみ設定（sourceされた場合に対応）
if [[ -z "${SCRIPT_DIR:-}" ]]; then
    declare -r SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fi
if [[ -z "${AGENT_ROOT:-}" ]]; then
    declare -r AGENT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
fi

# Load dependencies
source "$SCRIPT_DIR/../utils/logger.sh" 2>/dev/null || {
    # Fallback logger if not available
    log() { echo "[$1] $2" >&2; }
    log_error_with_trace() { log ERROR "$1"; }
}

# Configuration paths（読み取り専用）
declare -r CONFIG_DIR="${AGENT_ROOT}/config"
declare -r OPTION_PATTERNS_FILE="${CONFIG_DIR}/option-patterns.yaml"
declare -r NUMERIC_CONSTRAINTS_FILE="${CONFIG_DIR}/numeric-constraints.yaml"
declare -r ENUM_DEFINITIONS_FILE="${CONFIG_DIR}/enum-definitions.yaml"

# Global variables for caching
declare -g OPTION_PATTERNS_CACHE=""
declare -g NUMERIC_CONSTRAINTS_CACHE=""
declare -g ENUM_DEFINITIONS_CACHE=""

#######################################
# Check if yq is installed
# Globals:
#   None
# Arguments:
#   None
# Returns:
#   0 - yq is available, 1 - not available
#######################################
check_yq_installation() {
    if command -v yq &>/dev/null; then
        local yq_version
        yq_version=$(yq --version 2>&1 | head -1)
        log DEBUG "yq detected: $yq_version"
        return 0
    else
        log ERROR "yq is not installed. Install with:"
        log ERROR "  macOS:  brew install yq"
        log ERROR "  Ubuntu: snap install yq"
        log ERROR "  https://github.com/mikefarah/yq"
        return 1
    fi
}

#######################################
# Load option pattern definitions from YAML
# Globals:
#   OPTION_PATTERNS_CACHE (sets)
# Arguments:
#   $1 - Pattern file path (optional, defaults to OPTION_PATTERNS_FILE)
# Returns:
#   0 - success, 1 - failure
#######################################
load_option_patterns() {
    local pattern_file="${1:-$OPTION_PATTERNS_FILE}"

    # Return cached data if available
    if [[ -n "$OPTION_PATTERNS_CACHE" ]]; then
        log DEBUG "Using cached option patterns"
        return 0
    fi

    # File existence check
    if [[ ! -f "$pattern_file" ]]; then
        log ERROR "Pattern file not found: $pattern_file"
        return 1
    fi

    # Check yq availability
    if ! check_yq_installation; then
        return 1
    fi

    # Load YAML and convert to JSON for easier processing
    if ! OPTION_PATTERNS_CACHE=$(yq -o=json '.' "$pattern_file" 2>/dev/null); then
        log ERROR "Failed to parse pattern file: $pattern_file"
        return 1
    fi

    log INFO "Loaded option patterns from: $pattern_file"
    return 0
}

#######################################
# Infer option type from name (data-driven version)
# Globals:
#   OPTION_PATTERNS_CACHE (reads)
# Arguments:
#   $1 - Option name (e.g., --port, --input)
#   $2 - Option description (optional)
# Returns:
#   Inferred type (numeric|path|enum|boolean|string)
#######################################
infer_option_type() {
    local option_name="$1"
    local option_description="${2:-}"

    # Ensure patterns are loaded
    if [[ -z "$OPTION_PATTERNS_CACHE" ]]; then
        if ! load_option_patterns; then
            echo "string"  # Fallback to string type
            return 0
        fi
    fi

    # Normalize option name (remove hyphens, lowercase)
    # Performance: use Bash built-ins instead of external commands
    local normalized_name="$option_name"
    # Remove leading -- or -
    normalized_name="${normalized_name#--}"
    normalized_name="${normalized_name#-}"
    # Convert to lowercase
    normalized_name="${normalized_name,,}"
    # Replace hyphens with underscores
    normalized_name="${normalized_name//-/_}"

    # Pattern matching (priority order)
    local matched_type
    matched_type=$(echo "$OPTION_PATTERNS_CACHE" | jq -r --arg name "$normalized_name" '
        .patterns
        | sort_by(.priority) | reverse
        | .[]
        | select(.keywords | map(. as $kw | $name | contains($kw)) | any)
        | .type
        | select(. != null)
    ' 2>/dev/null | head -1)

    # Return matched type or default
    if [[ -n "$matched_type" ]]; then
        echo "$matched_type"
        log DEBUG "Inferred type for '$option_name': $matched_type"
    else
        echo "string"
        log DEBUG "No pattern matched for '$option_name', defaulting to string"
    fi

    return 0
}

#######################################
# Extract numeric constraints from configuration
# Globals:
#   NUMERIC_CONSTRAINTS_CACHE (reads/sets)
# Arguments:
#   $1 - Option name
# Returns:
#   JSON format constraint information
#######################################
extract_numeric_constraints() {
    local option_name="$1"

    # Load constraints if not cached
    if [[ -z "$NUMERIC_CONSTRAINTS_CACHE" ]]; then
        if [[ -f "$NUMERIC_CONSTRAINTS_FILE" ]]; then
            if check_yq_installation; then
                NUMERIC_CONSTRAINTS_CACHE=$(yq -o=json '.' "$NUMERIC_CONSTRAINTS_FILE" 2>/dev/null) || true
            fi
        fi
    fi

    # Normalize option name
    # Performance: use Bash built-ins instead of external commands
    local normalized_name="$option_name"
    normalized_name="${normalized_name#--}"
    normalized_name="${normalized_name#-}"
    normalized_name="${normalized_name,,}"
    normalized_name="${normalized_name//-/_}"

    # Search in YAML configuration
    local constraint_json
    if [[ -n "$NUMERIC_CONSTRAINTS_CACHE" ]]; then
        constraint_json=$(echo "$NUMERIC_CONSTRAINTS_CACHE" | jq -c --arg name "$normalized_name" '
            .constraints
            | to_entries
            | .[]
            | select(.value.aliases | map(. == $name or ($name | contains(.))) | any)
            | .value
        ' 2>/dev/null | head -1)
    fi

    # Return matched constraints or default
    if [[ -n "$constraint_json" && "$constraint_json" != "null" ]]; then
        echo "$constraint_json"
        log DEBUG "Found constraints for '$option_name'"
    else
        # Default constraints
        echo '{"min": 0, "max": 2147483647, "type": "integer", "unit": null}'
        log DEBUG "Using default constraints for '$option_name'"
    fi

    return 0
}

#######################################
# Extract enumeration values from configuration
# Globals:
#   ENUM_DEFINITIONS_CACHE (reads/sets)
# Arguments:
#   $1 - Option name
#   $2 - Help text (optional, for dynamic extraction)
# Returns:
#   JSON array of allowed values
#######################################
extract_enum_values() {
    local option_name="$1"
    local help_text="${2:-}"

    # Load enum definitions if not cached
    if [[ -z "$ENUM_DEFINITIONS_CACHE" ]]; then
        if [[ -f "$ENUM_DEFINITIONS_FILE" ]]; then
            if check_yq_installation; then
                ENUM_DEFINITIONS_CACHE=$(yq -o=json '.' "$ENUM_DEFINITIONS_FILE" 2>/dev/null) || true
            fi
        fi
    fi

    # Normalize option name
    # Performance: use Bash built-ins instead of external commands
    local normalized_name="$option_name"
    normalized_name="${normalized_name#--}"
    normalized_name="${normalized_name#-}"
    normalized_name="${normalized_name,,}"
    normalized_name="${normalized_name//-/_}"

    # Search in YAML configuration
    local enum_json
    if [[ -n "$ENUM_DEFINITIONS_CACHE" ]]; then
        enum_json=$(echo "$ENUM_DEFINITIONS_CACHE" | jq -c --arg name "$normalized_name" '
            .enums
            | to_entries
            | .[]
            | select(.value.aliases | map(. == $name or ($name | contains(.))) | any)
            | .value.values
        ' 2>/dev/null | head -1)
    fi

    # Return matched enum values
    if [[ -n "$enum_json" && "$enum_json" != "null" ]]; then
        echo "$enum_json"
        log DEBUG "Found enum values for '$option_name'"
        return 0
    fi

    # Fallback: extract from help text (with security constraints)
    if [[ -n "$help_text" ]]; then
        # Security: limit input length to prevent ReDoS
        # Performance: use Bash built-in substring instead of head
        local safe_help_text="${help_text:0:1000}"

        # Extract pattern like [value1|value2|value3]
        # Performance: use Bash regex instead of grep pipeline
        if [[ "$safe_help_text" =~ \[([a-zA-Z0-9|_-]{1,100})\] ]]; then
            local bracket_content="${BASH_REMATCH[1]}"

            # Split by pipe and convert to JSON array
            # Performance: use Bash array instead of tr|jq pipeline
            local -a values
            IFS='|' read -ra values <<< "$bracket_content"

            # Build JSON array manually (faster than jq for small arrays)
            local json_array="["
            local first=1
            for value in "${values[@]}"; do
                [[ -z "$value" ]] && continue
                if [[ $first -eq 1 ]]; then
                    json_array+="\"$value\""
                    first=0
                else
                    json_array+=",\"$value\""
                fi
            done
            json_array+="]"

            echo "$json_array"
            log DEBUG "Extracted enum values from help text for '$option_name'"
            return 0
        fi
    fi

    # Default: empty array
    echo '[]'
    log DEBUG "No enum values found for '$option_name'"
    return 0
}

#######################################
# Validate CLI binary path (security check)
# Arguments:
#   $1 - CLI binary path
# Returns:
#   0 - valid, 1 - invalid
#######################################
validate_cli_binary() {
    local binary="$1"

    # Absolute path check
    if [[ ! "$binary" =~ ^/ ]]; then
        log ERROR "CLI binary must be absolute path: $binary"
        return 1
    fi

    # Executable file check
    if [[ ! -x "$binary" ]]; then
        log ERROR "CLI binary is not executable: $binary"
        return 1
    fi

    # Path traversal check (using realpath)
    local real_binary
    if command -v realpath &>/dev/null; then
        real_binary=$(realpath -s "$binary" 2>/dev/null) || return 1
    elif command -v greadlink &>/dev/null; then
        # macOS alternative
        real_binary=$(greadlink -f "$binary" 2>/dev/null) || return 1
    else
        # Fallback: basic check
        real_binary="$binary"
    fi

    # Whitelist check (warning only)
    if [[ ! "$real_binary" =~ ^(/bin/|/usr/bin/|/usr/local/bin/|/opt/) ]]; then
        log WARN "CLI binary outside standard paths: $real_binary"
        log WARN "Proceed with caution"
    fi

    log INFO "Validated CLI binary: $real_binary"
    return 0
}

#######################################
# Analyze options from CLI analysis JSON
# Arguments:
#   $1 - Analysis JSON file path
# Returns:
#   JSON with option types and constraints
#######################################
analyze_option_types() {
    local analysis_json="$1"

    if [[ ! -f "$analysis_json" ]]; then
        log ERROR "Analysis JSON not found: $analysis_json"
        return 1
    fi

    # Load option patterns
    if ! load_option_patterns; then
        log ERROR "Failed to load option patterns"
        return 1
    fi

    # Extract options from analysis JSON
    local options
    options=$(jq -r '.options[]?' "$analysis_json" 2>/dev/null)

    if [[ -z "$options" ]]; then
        log WARN "No options found in analysis JSON"
        echo '{}'
        return 0
    fi

    # Analyze each option
    local result='{"numeric": [], "path": [], "enum": [], "boolean": [], "string": []}'

    while IFS= read -r option; do
        [[ -z "$option" ]] && continue

        local option_type
        option_type=$(infer_option_type "$option")

        # Add to appropriate category
        result=$(echo "$result" | jq --arg opt "$option" --arg type "$option_type" \
            '.[$type] += [$opt]')
    done <<< "$options"

    echo "$result"
    return 0
}

# Main execution (if run directly)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 <command> [args...]"
        echo "Commands:"
        echo "  infer-type <option-name>           - Infer option type"
        echo "  extract-constraints <option-name>  - Extract numeric constraints"
        echo "  extract-enum <option-name>         - Extract enum values"
        echo "  validate-binary <binary-path>      - Validate CLI binary"
        echo "  analyze-json <analysis.json>       - Analyze all options"
        exit 1
    fi

    command="$1"
    shift

    case "$command" in
        infer-type)
            infer_option_type "$@"
            ;;
        extract-constraints)
            extract_numeric_constraints "$@"
            ;;
        extract-enum)
            extract_enum_values "$@"
            ;;
        validate-binary)
            validate_cli_binary "$@"
            ;;
        analyze-json)
            analyze_option_types "$@"
            ;;
        *)
            echo "Unknown command: $command"
            exit 1
            ;;
    esac
fi
