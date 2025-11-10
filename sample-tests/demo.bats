#!/usr/bin/env bats
#
# Demo test suite for HTML report generation
#

@test "Basic echo command works" {
    run echo "Hello, World!"
    [ "$status" -eq 0 ]
    [ "$output" = "Hello, World!" ]
}

@test "ls command lists files" {
    run ls /tmp
    [ "$status" -eq 0 ]
}

@test "grep finds pattern in text" {
    run bash -c 'echo "test string" | grep "test"'
    [ "$status" -eq 0 ]
}

@test "File creation and deletion" {
    tmpfile=$(mktemp)
    echo "content" > "$tmpfile"
    [ -f "$tmpfile" ]
    rm "$tmpfile"
    [ ! -f "$tmpfile" ]
}

@test "Environment variable expansion" {
    export TEST_VAR="test_value"
    run bash -c 'echo $TEST_VAR'
    [ "$status" -eq 0 ]
    [ "$output" = "test_value" ]
}

@test "Command substitution works" {
    result=$(echo "substituted")
    [ "$result" = "substituted" ]
}

@test "Arithmetic operations" {
    result=$((5 + 3))
    [ "$result" -eq 8 ]
}

@test "String concatenation" {
    str1="Hello"
    str2="World"
    result="${str1} ${str2}"
    [ "$result" = "Hello World" ]
}

@test "File permissions check" {
    tmpfile=$(mktemp)
    chmod 644 "$tmpfile"
    run ls -l "$tmpfile"
    [ "$status" -eq 0 ]
    rm "$tmpfile"
}

@test "Pipeline operations" {
    run bash -c 'echo "abc" | tr "a-z" "A-Z"'
    [ "$status" -eq 0 ]
    [ "$output" = "ABC" ]
}

@test "Conditional execution" {
    run bash -c 'true && echo "success"'
    [ "$status" -eq 0 ]
    [ "$output" = "success" ]
}

@test "Loop iteration" {
    count=0
    for i in {1..5}; do
        count=$((count + 1))
    done
    [ "$count" -eq 5 ]
}

@test "Array operations" {
    arr=("one" "two" "three")
    [ "${arr[0]}" = "one" ]
    [ "${#arr[@]}" -eq 3 ]
}

@test "Function definition and call" {
    test_func() {
        echo "function called"
    }
    result=$(test_func)
    [ "$result" = "function called" ]
}

@test "Skip example - intentionally skipped" {
    skip "This test is intentionally skipped for demo"
    false
}
