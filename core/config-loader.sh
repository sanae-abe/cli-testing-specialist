#!/usr/bin/env bash
#
# config-loader.sh - YAML設定読み込みとバリデーション
# CLI Testing Specialist Agent v1.1.0
#
# 機能:
# - YAMLファイル読み込み（yq優先、Python fallback）
# - 環境変数による設定オーバーライド
# - スキーマバリデーション
# - デフォルト値適用

set -euo pipefail
IFS=$'\n\t'

# スクリプトのディレクトリを取得
CONFIG_LOADER_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# デフォルト設定パス
DEFAULT_CONFIG="$CONFIG_LOADER_ROOT/config/default.yaml"
DEFAULT_SCHEMA="$CONFIG_LOADER_ROOT/config/schema.yaml"

# ロガーの読み込み（存在する場合）
if [[ -f "$CONFIG_LOADER_ROOT/utils/logger.sh" ]]; then
    source "$CONFIG_LOADER_ROOT/utils/logger.sh"
else
    # ロガー未使用時のフォールバック
    log() { echo "[$1] ${@:2}" >&2; }
fi

# YAML読み込み方法の検出
detect_yaml_parser() {
    if command -v yq &>/dev/null; then
        echo "yq"
    elif command -v python3 &>/dev/null; then
        # PyYAMLが利用可能か確認
        if python3 -c "import yaml" 2>/dev/null; then
            echo "python"
        else
            echo "none"
        fi
    else
        echo "none"
    fi
}

# yqを使用したYAML値読み込み
read_yaml_with_yq() {
    local yaml_file="$1"
    local key_path="$2"

    if [[ ! -f "$yaml_file" ]]; then
        log ERROR "YAML file not found: $yaml_file"
        return 1
    fi

    # yqでキーパスを読み込み（.で区切られたパス対応）
    local raw_value
    raw_value=$(yq eval ".${key_path}" "$yaml_file" 2>/dev/null || echo "null")

    # 配列の場合はカンマ区切りに変換
    if [[ "$raw_value" =~ ^\[.*\]$ ]]; then
        # JSON配列をカンマ区切りに変換
        echo "$raw_value" | yq eval '.[]' - 2>/dev/null | paste -sd ',' - || echo "null"
    else
        echo "$raw_value"
    fi
}

# Pythonを使用したYAML値読み込み
read_yaml_with_python() {
    local yaml_file="$1"
    local key_path="$2"

    if [[ ! -f "$yaml_file" ]]; then
        log ERROR "YAML file not found: $yaml_file"
        return 1
    fi

    python3 <<EOF
import yaml
import sys

try:
    with open("$yaml_file", "r") as f:
        data = yaml.safe_load(f)

    keys = "$key_path".split(".")
    value = data

    for key in keys:
        if isinstance(value, dict) and key in value:
            value = value[key]
        else:
            print("null")
            sys.exit(0)

    if isinstance(value, list):
        print(",".join(str(v) for v in value))
    elif isinstance(value, bool):
        print(str(value).lower())
    else:
        print(value)
except Exception as e:
    print("null", file=sys.stderr)
    sys.exit(1)
EOF
}

# YAML値読み込み統合関数
read_config_value() {
    local key_path="$1"
    local config_file="${2:-$DEFAULT_CONFIG}"

    local parser
    parser=$(detect_yaml_parser)

    case "$parser" in
        yq)
            read_yaml_with_yq "$config_file" "$key_path"
            ;;
        python)
            read_yaml_with_python "$config_file" "$key_path"
            ;;
        none)
            log ERROR "No YAML parser available (install yq or python3 with PyYAML)"
            return 1
            ;;
    esac
}

# 環境変数オーバーライドの適用
apply_env_overrides() {
    local key_path="$1"
    local default_value="$2"

    # schema.yamlから環境変数マッピングを取得
    local env_mappings
    env_mappings=$(read_config_value "environment_overrides.mappings" "$DEFAULT_SCHEMA")

    # 現在のkey_pathに対応する環境変数を検索
    local env_var
    case "$key_path" in
        "test_modules")
            env_var="${CLI_TEST_MODULES:-}"
            ;;
        "report_format")
            env_var="${CLI_TEST_REPORT_FORMAT:-}"
            ;;
        "docker.enabled")
            env_var="${CLI_TEST_DOCKER_ENABLED:-}"
            ;;
        "docker.environments")
            env_var="${CLI_TEST_DOCKER_ENVIRONMENTS:-}"
            ;;
        "logging.level")
            env_var="${CLI_TEST_LOG_LEVEL:-}"
            ;;
        "output.base_dir")
            env_var="${CLI_TEST_OUTPUT_DIR:-}"
            ;;
        "timeouts.test_execution")
            env_var="${CLI_TEST_TIMEOUT:-}"
            ;;
        "timeouts.docker_container")
            env_var="${CLI_TEST_DOCKER_TIMEOUT:-}"
            ;;
        *)
            env_var=""
            ;;
    esac

    # 環境変数が設定されている場合は優先
    if [[ -n "$env_var" ]]; then
        echo "$env_var"
    else
        echo "$default_value"
    fi
}

# 設定値の検証
validate_config_value() {
    local key_path="$1"
    local value="$2"

    # スキーマから許可値を取得（配列形式で取得）
    local schema_key="${key_path}"
    local allowed_values_raw
    allowed_values_raw=$(yq eval ".${schema_key}.allowed_values[]" "$DEFAULT_SCHEMA" 2>/dev/null || echo "")

    if [[ -z "$allowed_values_raw" ]]; then
        # 許可値が定義されていない場合はスキップ
        return 0
    fi

    # 許可値をカンマ区切りに変換
    local allowed_values
    allowed_values=$(echo "$allowed_values_raw" | paste -sd ',' -)

    # 配列値の検証（カンマ区切り）
    if [[ "$value" == *","* ]]; then
        IFS=',' read -ra values <<< "$value"
        for v in "${values[@]}"; do
            # 前後の空白を削除
            v=$(echo "$v" | xargs)
            if ! echo "$allowed_values" | grep -qw "$v"; then
                log ERROR "Invalid value '$v' for $key_path. Allowed: $allowed_values"
                return 1
            fi
        done
    else
        if ! echo "$allowed_values" | grep -qw "$value"; then
            log ERROR "Invalid value '$value' for $key_path. Allowed: $allowed_values"
            return 1
        fi
    fi

    return 0
}

# 設定値の取得（デフォルト→環境変数の順で優先）
get_config() {
    local key_path="$1"
    local custom_config="${2:-}"

    # カスタム設定ファイルが指定されている場合
    local config_file="$DEFAULT_CONFIG"
    if [[ -n "$custom_config" ]] && [[ -f "$custom_config" ]]; then
        config_file="$custom_config"
        log DEBUG "Using custom config: $custom_config"
    fi

    # デフォルト値を読み込み
    local default_value
    default_value=$(read_config_value "$key_path" "$config_file")

    if [[ "$default_value" == "null" ]]; then
        log ERROR "Config key not found: $key_path"
        return 1
    fi

    # 環境変数オーバーライドを適用
    local final_value
    final_value=$(apply_env_overrides "$key_path" "$default_value")

    # バリデーション実行
    validate_config_value "$key_path" "$final_value" || return 1

    echo "$final_value"
}

# 設定全体のロードとエクスポート
load_all_config() {
    local custom_config="${1:-}"

    log INFO "Loading configuration"

    # YAML parserの確認
    local parser
    parser=$(detect_yaml_parser)
    log DEBUG "YAML parser: $parser"

    if [[ "$parser" == "none" ]]; then
        log ERROR "No YAML parser available. Install yq or python3 with PyYAML"
        return 1
    fi

    # 主要設定値をエクスポート
    export CONFIG_TEST_MODULES
    CONFIG_TEST_MODULES=$(get_config "test_modules" "$custom_config")

    export CONFIG_REPORT_FORMAT
    CONFIG_REPORT_FORMAT=$(get_config "report_format" "$custom_config")

    export CONFIG_TIMEOUT_TEST
    CONFIG_TIMEOUT_TEST=$(get_config "timeouts.test_execution" "$custom_config")

    export CONFIG_TIMEOUT_DOCKER
    CONFIG_TIMEOUT_DOCKER=$(get_config "timeouts.docker_container" "$custom_config")

    export CONFIG_TIMEOUT_ANALYSIS
    CONFIG_TIMEOUT_ANALYSIS=$(get_config "timeouts.cli_analysis" "$custom_config")

    export CONFIG_DOCKER_ENABLED
    CONFIG_DOCKER_ENABLED=$(get_config "docker.enabled" "$custom_config")

    export CONFIG_DOCKER_ENVIRONMENTS
    CONFIG_DOCKER_ENVIRONMENTS=$(get_config "docker.environments" "$custom_config")

    export CONFIG_SHELL_DETECTION_ENABLED
    CONFIG_SHELL_DETECTION_ENABLED=$(get_config "shell_detection.enabled" "$custom_config")

    export CONFIG_OUTPUT_BASE_DIR
    CONFIG_OUTPUT_BASE_DIR=$(get_config "output.base_dir" "$custom_config")

    export CONFIG_LOG_LEVEL
    CONFIG_LOG_LEVEL=$(get_config "logging.level" "$custom_config")

    export CONFIG_LOG_COLORS
    CONFIG_LOG_COLORS=$(get_config "logging.enable_colors" "$custom_config")

    log INFO "Configuration loaded successfully"
    log DEBUG "  Test Modules: $CONFIG_TEST_MODULES"
    log DEBUG "  Report Format: $CONFIG_REPORT_FORMAT"
    log DEBUG "  Output Dir: $CONFIG_OUTPUT_BASE_DIR"
    log DEBUG "  Log Level: $CONFIG_LOG_LEVEL"

    return 0
}

# 設定ダンプ（デバッグ用）
dump_config() {
    cat <<EOF
=== CLI Testing Specialist Configuration ===
Test Modules: ${CONFIG_TEST_MODULES:-not loaded}
Report Format: ${CONFIG_REPORT_FORMAT:-not loaded}
Timeouts:
  - Test Execution: ${CONFIG_TIMEOUT_TEST:-not loaded}s
  - Docker: ${CONFIG_TIMEOUT_DOCKER:-not loaded}s
  - Analysis: ${CONFIG_TIMEOUT_ANALYSIS:-not loaded}s
Docker:
  - Enabled: ${CONFIG_DOCKER_ENABLED:-not loaded}
  - Environments: ${CONFIG_DOCKER_ENVIRONMENTS:-not loaded}
Shell Detection: ${CONFIG_SHELL_DETECTION_ENABLED:-not loaded}
Output Directory: ${CONFIG_OUTPUT_BASE_DIR:-not loaded}
Logging:
  - Level: ${CONFIG_LOG_LEVEL:-not loaded}
  - Colors: ${CONFIG_LOG_COLORS:-not loaded}
===========================================
EOF
}

# スクリプト直接実行時のテスト
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "=== Config Loader Test ==="

    # YAML parser検出
    echo "YAML Parser: $(detect_yaml_parser)"

    # 設定値取得テスト
    echo "Test Modules: $(get_config 'test_modules')"
    echo "Report Format: $(get_config 'report_format')"
    echo "Docker Enabled: $(get_config 'docker.enabled')"
    echo "Log Level: $(get_config 'logging.level')"

    # 全設定ロード
    load_all_config

    # 設定ダンプ
    dump_config
fi
