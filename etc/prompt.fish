set -g fish_transient_prompt 1

function fish_prompt
    /Users/chris/src/my-prompt/target/release/my-prompt --code $status $argv
end
